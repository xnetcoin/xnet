const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

async function main() {
    console.log("🔥 Xnetcoin Spam-Bot ishga tushdi!");
    
    // Tarmog'ingizga (Alice'ning portiga) ulanish
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

    // Hamyonlarni tayyorlash
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');

    // Alice'ning hozirgi tranzaksiya raqamini (nonce) bilib olamiz
    let { nonce } = await api.query.system.account(alice.address);
    let currentNonce = nonce.toNumber();

    console.log(`🚀 Hujum boshlanmoqda! TPS zoriqishi yaratilyapti...`);
    
    const TRANZAKSIYALAR_SONI = 10000; // Qancha jo'natish (10 ming)
    
    for (let i = 0; i < TRANZAKSIYALAR_SONI; i++) {
        // Bobga 1 MICROUNIT (juda mayda chaqa) yuborish
        const transfer = api.tx.balances.transferKeepAlive(bob.address, 1000000000000); 

        // Tizim kutib o'tirmasligi uchun nonce ni qo'lda oshirib, ketma-ket otib tashlaymiz
        transfer.signAndSend(alice, { nonce: currentNonce });
        currentNonce++;

        if (i % 1000 === 0 && i !== 0) {
            console.log(`💥 ${i} ta tranzaksiya tarmoqqa otildi...`);
        }
    }

    console.log(`✅ Hamma ${TRANZAKSIYALAR_SONI} ta tranzaksiya Muvaffaqiyatli tarmoqqa yuborildi!`);
    console.log(`Tezda Polkadot.js Explorer'ga kiring va "Fee" narxlarining qimmatlashishini kuzating!`);
    
    // Skript darhol o'chib qolmasligi uchun ozgina kutamiz
    setTimeout(() => process.exit(), 30000);
}

main().catch(console.error);
