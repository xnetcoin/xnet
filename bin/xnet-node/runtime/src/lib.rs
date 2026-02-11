#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::prelude::*;
use sp_core::OpaqueMetadata;
use sp_core::Encode;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys, KeyTypeId,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, Verify,
        OpaqueKeys, Convert,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature, Perbill, Permill,
};
use sp_api::impl_runtime_apis;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem,
        EnsureOrigin,
        tokens::Pay, 
    },
    weights::{
        constants::{
          RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::EnsureRoot;
use frame_election_provider_support::{
    onchain, 
    SequentialPhragmen,    
};
use sp_staking::currency_to_vote::U128CurrencyToVote;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::ConstFeeMultiplier;

// --- TYPES ---
pub type BlockNumber = u32;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type Nonce = u32;
pub type Hash = sp_core::H256;

// --- OPAQUE TYPES ---
pub mod opaque {
    use super::*;
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub babe: BabeId,
            pub grandpa: GrandpaId,
        }
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("xnetcoin"),
    impl_name: create_runtime_str!("xnetcoin"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength::max_with_normal_ratio(
        5 * 1024 * 1024,
        NORMAL_DISPATCH_RATIO,
    );
    pub const SS58Prefix: u8 = 42;
}

// --- TOKENOMICS ---
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const UNIT: Balance = 1_000_000_000_000_000_000;
pub const MILLIUNIT: Balance = UNIT / 1_000;
pub const MICROUNIT: Balance = MILLIUNIT / 1_000;

pub const BLOCK_REWARD: Balance = 1_117 * MILLIUNIT;
pub const MIN_VALIDATOR_BOND: Balance = 8_000 * UNIT;
pub const MIN_NOMINATOR_BOND: Balance = 1_000 * UNIT;
pub const EXISTENTIAL_DEPOSIT: Balance = 1 * MILLIUNIT;

// --- PALLET CONFIGS ---

impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    type AccountId = AccountId;
    type RuntimeCall = RuntimeCall;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Block = Block;
    type RuntimeTask = RuntimeTask;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const EpochDuration: u64 = (1 * HOURS) as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, BabeId)>>::Proof;
    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<1000>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
    type MaxNominators = ConstU32<1000>;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeFreezeReason = ();
    type RuntimeHoldReason = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MICROUNIT;
    pub const FeeMultiplier: sp_runtime::FixedU128 = sp_runtime::FixedU128::from_u32(1);
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = Staking;
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

// --- STAKING ---
parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 24;
    pub const BondingDuration: sp_staking::EraIndex = 28;
    pub const SlashDeferDuration: sp_staking::EraIndex = 7;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const HistoryDepth: u32 = 84;
}

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
    type MaxNominators = ConstU32<1000>;
    type MaxValidators = ConstU32<1000>;
}

pub struct OnChainSeqPhragmen;


impl onchain::Config for OnChainSeqPhragmen {
    type System = Runtime;
    type Solver = SequentialPhragmen<AccountId, sp_runtime::Perbill>;
    type DataProvider = Staking;
    type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
    type MaxWinners = ConstU32<100>;
    type Bounds = DefaultElectionBounds;
}
impl pallet_staking::Config for Runtime {
    type Currency = Balances;
    type CurrencyBalance = Balance;
    type UnixTime = Timestamp;
    type RuntimeEvent = RuntimeEvent;
    type Slash = Treasury; 
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type AdminOrigin = frame_system::EnsureRoot<AccountId>;
    type SessionInterface = Self;
	type EraPayout = ();
    type NextNewSession = Session;
    type MaxExposurePageSize = ConstU32<256>;
    type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Runtime>; 
    type TargetList = pallet_staking::UseValidatorsMap<Runtime>;
    type NominationsQuota = pallet_staking::FixedNominationsQuota<16>;
    type MaxUnlockingChunks = ConstU32<32>;
    type HistoryDepth = HistoryDepth;
    type EventListeners = ();
    type BenchmarkingConfig = StakingBenchmarkingConfig;
    type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
    type CurrencyToVote = U128CurrencyToVote;
    type RewardRemainder = Treasury;
    type MaxControllersInDeprecationBatch = ConstU32<100>;
    type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy<17>; // FIX: () o'rniga
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = (Babe, Grandpa);
    type Keys = opaque::SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

// --- TREASURY & SUDO ---

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 100 * UNIT;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const MaxApprovals: u32 = 100;
    pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;

    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const TreasuryAccount: AccountId = AccountId::new([0u8; 32]); 
}

pub struct SimplePay;
impl Pay for SimplePay {
    type Balance = Balance;
    type Beneficiary = AccountId;
    type AssetKind = ();
    type Id = u64; // To'lov identifikatori turi
    type Error = sp_runtime::DispatchError; // Xatolik turi

    fn pay(
        _who: &Self::Beneficiary,
        _asset: Self::AssetKind,
        _amount: Self::Balance,
    ) -> Result<Self::Id, Self::Error> {
        Ok(0) 
    }

    fn check_payment(_id: Self::Id) -> frame_support::traits::tokens::pay::PaymentStatus {
        frame_support::traits::tokens::pay::PaymentStatus::Success
    }
}

pub struct RootSpender;
impl EnsureOrigin<RuntimeOrigin> for RootSpender {
    type Success = Balance;
    fn try_origin(o: RuntimeOrigin) -> Result<Balance, RuntimeOrigin> {
        EnsureRoot::try_origin(o).map(|_| Balance::MAX)
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        EnsureRoot::try_successful_origin()
    }
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type AssetKind = ();
    type Beneficiary = AccountId;
    type BeneficiaryLookup = AccountIdLookup<AccountId, ()>;
    type Paymaster = frame_support::traits::tokens::pay::PayFromAccount<Balances, TreasuryAccount>;
    type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
    type PayoutPeriod = ConstU32<60>;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = ();
    type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
    type RejectOrigin = frame_system::EnsureRoot<AccountId>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}


// --- ELECTION PROVIDER ---

frame_election_provider_support::generate_solution_type!(
    #[compact]
    pub struct NposSolution16::<
        VoterIndex = u32,
        TargetIndex = u16,
        Accuracy = sp_runtime::PerU16,
        // FIX: Syntax error fixed "::"
        MaxVoters = ConstU32::<2400>, 
    >(16)
);
parameter_types! {
    pub MaxElectionWeight: Weight = BlockWeights::get().max_block;
}

pub struct CustomMinerConfig;
impl pallet_election_provider_multi_phase::MinerConfig for CustomMinerConfig {
    type AccountId = AccountId;
    type MaxLength = ConstU32<256>;
    type MaxWeight = MaxElectionWeight;
    type MaxVotesPerVoter = ConstU32<16>;
    type MaxWinners = ConstU32<100>;
    type Solution = NposSolution16;
    fn solution_weight(_v: u32, _t: u32, _a: u32, _d: u32) -> Weight {
        Weight::zero()
    }
}

pub struct DepositBase;
impl Convert<usize, Balance> for DepositBase {
    fn convert(x: usize) -> Balance {
        (x as u128) * UNIT
    }
}


// ---YANGI BO'LIM---
pub struct ElectionMultiPhaseBenchmarkConfig;

impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionMultiPhaseBenchmarkConfig {
    const VOTERS: [u32; 2] = [1000, 2000]; 
    const TARGETS: [u32; 2] = [100, 200];
    const ACTIVE_VOTERS: [u32; 2] = [500, 1000];
    const DESIRED_TARGETS: [u32; 2] = [50, 100];
    
    // MISSING ITEMS:
    const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
    const MINER_MAXIMUM_VOTERS: u32 = 1000;
    const MAXIMUM_TARGETS: u32 = 300;
}

impl pallet_election_provider_multi_phase::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EstimateCallFee = TransactionPayment;
    type SignedPhase = ();
    type UnsignedPhase = ();
    type SignedMaxSubmissions = ConstU32<10>;
    type SignedRewardBase = ConstU128<{ 1 * UNIT }>;
    type SignedDepositBase = DepositBase;
    type SignedDepositByte = ConstU128<{ 1 * MICROUNIT }>;
    type SignedDepositWeight = ();
    type SignedMaxWeight = ();
    type SlashHandler = ();
    type RewardHandler = ();
    type BetterSignedThreshold = ();
    type OffchainRepeat = ();
    type MinerTxPriority = ConstU64<{ u64::MAX }>;
    type MinerConfig = CustomMinerConfig;
    type Solver = SequentialPhragmen<AccountId, pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>>;
    type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Runtime>;
    type MaxWinners = ConstU32<100>;
    type ElectionBounds = DefaultElectionBounds;
    type SignedMaxRefunds = ConstU32<0>;
    type DataProvider = Staking;
    type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type BenchmarkingConfig = ElectionMultiPhaseBenchmarkConfig;
}	

// --- CUSTOM: Block Reward ---
parameter_types! {
    pub const InitialBlockReward: Balance = BLOCK_REWARD; 
    pub const HalvingInterval: u32 = 21_038_400;
}

impl pallet_block_reward::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type InitialBlockReward = InitialBlockReward; 
    type HalvingInterval = HalvingInterval;
}

parameter_types! {
    pub const DefaultElectionBounds: frame_election_provider_support::bounds::ElectionBounds = 
        frame_election_provider_support::bounds::ElectionBounds {
            voters: frame_election_provider_support::DataProviderBounds { 
                count: Some(frame_election_provider_support::bounds::CountBound(2500)), 
                size: Some(frame_election_provider_support::bounds::SizeBound(u32::MAX)) 
            },
            
            targets: frame_election_provider_support::DataProviderBounds { 
                count: Some(frame_election_provider_support::bounds::CountBound(100)), 
                size: Some(frame_election_provider_support::bounds::SizeBound(u32::MAX)) 
            }, 
        };
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>, 
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

// --- CONSTRUCT RUNTIME ---
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Babe: pallet_babe,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Authorship: pallet_authorship,
        Offences: pallet_offences,
        Staking: pallet_staking,
        Session: pallet_session,
        Historical: pallet_session::historical::{Pallet},
        Treasury: pallet_treasury,
        Sudo: pallet_sudo,
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
        BlockReward: pallet_block_reward,
    }
);

pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;
pub use pallet_staking::StakerStatus;
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: (1, 4),
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
    };

// --- RUNTIME API ---

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion { VERSION }
        fn execute_block(block: Block) { Executive::execute_block(block); }
        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
             Executive::initialize_block(header)
        }
    }
    
    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }
        fn finalize_block() -> <Block as BlockT>::Header { Executive::finalize_block() }
        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }
        fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) { Executive::offchain_worker(header) }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(sp_consensus_babe::BabeEpochConfiguration {
                c: (1, 4),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            });
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
            }
        }
        fn current_epoch_start() -> sp_consensus_babe::Slot { Babe::current_epoch_start() }
        fn current_epoch() -> sp_consensus_babe::Epoch { Babe::current_epoch() }
        fn next_epoch() -> sp_consensus_babe::Epoch { Babe::next_epoch() }
        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }
        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;
            Babe::submit_unsigned_equivocation_report(equivocation_proof, key_owner_proof)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }
        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList { Grandpa::grandpa_authorities() }
        fn current_set_id() -> sp_consensus_grandpa::SetId { Grandpa::current_set_id() }
        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;
            Grandpa::submit_unsigned_equivocation_report(equivocation_proof, key_owner_proof)
        }
        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            authority_id: sp_consensus_grandpa::AuthorityId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce { System::account_nonce(account) }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance { TransactionPayment::weight_to_fee(weight) }
        fn query_length_to_fee(length: u32) -> Balance { TransactionPayment::length_to_fee(length) }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec![]
        }
    }
}
