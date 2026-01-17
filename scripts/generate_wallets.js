const { Keypair } = require('@solana/web3.js');
const bs58 = require('bs58');
const fs = require('fs');

async function generate() {
    const wallets = [];
    for (let i = 0; i < 20; i++) {
        const kp = Keypair.generate();
        const encode = bs58.default ? bs58.default.encode : bs58.encode;
        wallets.push({
            pubkey: kp.publicKey.toString(),
            secret: encode(kp.secretKey)
        });
    }
    console.log(JSON.stringify(wallets, null, 2));
}

generate().catch(console.error);
