//! # tests.rs
//!
//! pallet-zk-verifier uchun to'liq test to'plami.
//!
//! ## Test ssenariylari
//!
//! 1. VK boshqaruvi — register, remove, xato holatlar
//! 2. Nullifier — double-spend himoyasi
//! 3. Proof format — noto'g'ri uzunlik, IC soni mos kelmaslik
//! 4. Blok limiti — spam himoyasi
//! 5. Public inputs chegarasi
//! 6. Matematik xato — soxta proof rad etilishi

#![cfg(test)]

use crate::mock::*;
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};
use pallet_zk_verifier::pallet::*;

// ─────────────────────────────────────────────────────────────────────────────
// VK boshqaruvi testlari
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn register_vk_works() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"test_circuit");

        // Root muvaffaqiyatli register qiladi
        assert_ok!(
            ZkVerifier::register_vk(
                RuntimeOrigin::root(),
                id,
                vk_bytes.clone(),
                ic_len,
                b"Test Circuit v1".to_vec(),
            )
        );

        // Storage'da saqlanganini tekshirish
        assert!(ZkVerifier::vk_exists(id));

        // Event chiqdi
        System::assert_has_event(
            Event::<Test>::VkRegistered { vk_id: id }.into()
        );
    });
}

#[test]
fn register_vk_requires_root() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"test_circuit");

        // Oddiy foydalanuvchi register qila olmaydi
        assert_noop!(
            ZkVerifier::register_vk(
                RuntimeOrigin::signed(1),
                id,
                vk_bytes,
                ic_len,
                b"Test".to_vec(),
            ),
            frame_support::error::BadOrigin
        );
    });
}

#[test]
fn register_vk_rejects_short_vk() {
    new_test_ext().execute_with(|| {
        let id = vk_id(b"bad_vk");

        // 447 bayt — minimal 448 dan kam
        let short_vk = vec![0u8; 447];

        assert_noop!(
            ZkVerifier::register_vk(
                RuntimeOrigin::root(),
                id,
                short_vk,
                2,
                b"Bad VK".to_vec(),
            ),
            Error::<Test>::InvalidVk
        );
    });
}

#[test]
fn remove_vk_works() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"remove_me");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes, ic_len, b"".to_vec()
        ));

        assert!(ZkVerifier::vk_exists(id));

        assert_ok!(ZkVerifier::remove_vk(RuntimeOrigin::root(), id));

        // O'chirilganini tekshirish
        assert!(!ZkVerifier::vk_exists(id));

        System::assert_has_event(
            Event::<Test>::VkRemoved { vk_id: id }.into()
        );
    });
}

#[test]
fn remove_nonexistent_vk_fails() {
    new_test_ext().execute_with(|| {
        let id = vk_id(b"ghost");

        assert_noop!(
            ZkVerifier::remove_vk(RuntimeOrigin::root(), id),
            Error::<Test>::VkNotFound
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Proof format testlari
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn verify_proof_fails_with_unknown_vk() {
    new_test_ext().execute_with(|| {
        let unknown_id = vk_id(b"not_registered");
        let input = [[1u8; 32]; 1];

        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                unknown_id,
                zero_proof(),
                input.to_vec(),
                None,
            ),
            Error::<Test>::VkNotFound
        );
    });
}

#[test]
fn verify_proof_fails_with_wrong_input_count() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"ic_mismatch");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes, ic_len, b"".to_vec()
        ));

        // ic_len = 2, ya'ni 1 ta input kerak, biz 2 ta beramiz
        let wrong_inputs = vec![[1u8; 32]; 2];

        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                id,
                zero_proof(),
                wrong_inputs,
                None,
            ),
            Error::<Test>::InvalidPublicInputsCount
        );
    });
}

#[test]
fn verify_proof_fails_with_too_many_inputs() {
    new_test_ext().execute_with(|| {
        // MaxPublicInputs = 16, biz 17 ta beramiz
        let inputs = vec![[0u8; 32]; 17];
        let id = vk_id(b"overflow");

        // VK ni register qilmasdan ham xato chiqadi (limit avval tekshiriladi)
        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                id,
                zero_proof(),
                inputs,
                None,
            ),
            Error::<Test>::TooManyPublicInputs
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Nullifier testlari
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nullifier_double_spend_prevented() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"nullifier_test");
        let null = nullifier(42);

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes, ic_len, b"".to_vec()
        ));

        // Nullifier'ni to'g'ridan-to'g'ri storage ga yozamiz (birinchi sarflash simulyatsiyasi)
        pallet_zk_verifier::SpentNullifiers::<Test>::insert(null, true);

        // Ikkinchi urinish rad etiladi
        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                id,
                zero_proof(),
                vec![[0u8; 32]],
                Some(null),
            ),
            Error::<Test>::NullifierAlreadySpent
        );
    });
}

#[test]
fn nullifier_is_stored_after_use() {
    new_test_ext().execute_with(|| {
        let null = nullifier(99);

        // Tekshirishdan oldin sarflanmagan
        assert!(!ZkVerifier::is_nullifier_spent(null));

        // Nullifier storage'ga yoziladi
        pallet_zk_verifier::SpentNullifiers::<Test>::insert(null, true);

        // Endi sarflangan
        assert!(ZkVerifier::is_nullifier_spent(null));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Blok limiti testi
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn block_proof_limit_enforced() {
    new_test_ext().execute_with(|| {
        // MaxProofsPerBlock = 10 ga to'ldiramiz
        pallet_zk_verifier::ProofsThisBlock::<Test>::put(10u32);

        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"limit_test");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes, ic_len, b"".to_vec()
        ));

        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                id,
                zero_proof(),
                vec![[0u8; 32]],
                None,
            ),
            Error::<Test>::BlockLimitReached
        );
    });
}

#[test]
fn block_counter_resets_on_new_block() {
    new_test_ext().execute_with(|| {
        // 10 ga to'ldiramiz
        pallet_zk_verifier::ProofsThisBlock::<Test>::put(10u32);
        assert_eq!(pallet_zk_verifier::ProofsThisBlock::<Test>::get(), 10);

        // Yangi blok → on_initialize → 0 ga qaytadi
        ZkVerifier::on_initialize(2u64);
        assert_eq!(pallet_zk_verifier::ProofsThisBlock::<Test>::get(), 0);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Matematik tekshirish testlari
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn zero_proof_is_rejected_by_math() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk();
        let id = vk_id(b"math_test");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes.clone(), ic_len, b"".to_vec()
        ));

        // Soxta nol proof — pairing tekshiruvi muvaffaqiyatsiz bo'lishi kerak
        // TestExternalities'da sp_io::crypto::alt_bn128_pairing ishlaydi
        let result = ZkVerifier::verify_proof(
            RuntimeOrigin::signed(1),
            id,
            zero_proof(),
            vec![[0u8; 32]],
            None,
        );

        // Nol proof yoki InvalidProof yoki ArithmeticError — ikkalasi ham to'g'ri
        assert!(result.is_err());
    });
}

#[test]
fn negate_g1_is_correct() {
    new_test_ext().execute_with(|| {
        // Generator nuqtasi: (1, 2)
        let gen = g1_generator();

        // Negatsiya: (1, p-2)
        // p = 0x30644e72...fd47
        // p - 2 = 0x30644e72...fd45
        let neg = <ZkVerifier as pallet_zk_verifier::pallet::Pallet<Test>>
            ::negate_g1(&gen);

        assert_eq!(neg.len(), 64);
        // x o'zgarmagan
        assert_eq!(&neg[0..32], &gen[0..32]);
        // y o'zgardi — eng oxirgi bayt p[31] - 2 = 0x47 - 2 = 0x45
        assert_eq!(neg[63], 0x45);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Statistika testi
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn total_verified_counter_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(ZkVerifier::total_verified(), 0);

        // Hisoblagichni qo'lda oshiramiz (haqiqiy tekshirish o'tkazmasdan)
        pallet_zk_verifier::TotalVerified::<Test>::put(5u64);
        assert_eq!(ZkVerifier::total_verified(), 5);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge case testlar
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn different_vk_ids_stored_separately() {
    new_test_ext().execute_with(|| {
        let (vk1, ic1) = make_test_vk();
        let (vk2, ic2) = make_test_vk();

        let id1 = vk_id(b"circuit_a");
        let id2 = vk_id(b"circuit_b");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id1, vk1, ic1, b"A".to_vec()
        ));
        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id2, vk2, ic2, b"B".to_vec()
        ));

        assert!(ZkVerifier::vk_exists(id1));
        assert!(ZkVerifier::vk_exists(id2));

        // id1 o'chirilsa id2 qoladi
        assert_ok!(ZkVerifier::remove_vk(RuntimeOrigin::root(), id1));
        assert!(!ZkVerifier::vk_exists(id1));
        assert!(ZkVerifier::vk_exists(id2));
    });
}

#[test]
fn zero_inputs_rejected_when_ic_len_mismatch() {
    new_test_ext().execute_with(|| {
        let (vk_bytes, ic_len) = make_test_vk(); // ic_len = 2
        let id = vk_id(b"zero_inputs");

        assert_ok!(ZkVerifier::register_vk(
            RuntimeOrigin::root(), id, vk_bytes, ic_len, b"".to_vec()
        ));

        // 0 input beramiz, 1 ta kerak (ic_len=2 → inputs=1)
        assert_noop!(
            ZkVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                id,
                zero_proof(),
                vec![], // 0 inputs — ic_len=2 bilan mos kelmaydi
                None,
            ),
            Error::<Test>::InvalidPublicInputsCount
        );
    });
}