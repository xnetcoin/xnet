const { ethers } = require("hardhat");

async function main() {
    // Bir blok kutish — timestamp sync uchun
    const provider = ethers.provider;
    console.log("Blok kutilmoqda...");
    
    const block1 = await provider.getBlockNumber();
    
    // 3 soniya kutish
    await new Promise(r => setTimeout(r, 3000));
    
    const block2 = await provider.getBlockNumber();
    console.log(`Blok: ${block1} → ${block2}`);
    console.log("Tayyor!");
}

main().catch(console.error);
