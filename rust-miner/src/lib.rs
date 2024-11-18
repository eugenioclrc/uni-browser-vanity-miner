use wasm_bindgen::prelude::*;

use sha3::{Keccak256, Digest};
use hex;

mod score;

fn compute_address(salt: &[u8; 32]) -> [u8; 20] {
    // uniswap bytecode hash, from https://blog.uniswap.org/uniswap-v4-address-mining-challenge
    const bytecode_hash: [u8; 32] = [
        0x94, 0xd1, 0x14, 0x29, 0x6a, 0x5a, 0xf8, 0x5c, 0x1f, 0xd2, 0xdc, 0x03, 0x9c, 0xda, 0xa3, 0x2f, 
        0x1e, 0xd4, 0xb0, 0xfe, 0x08, 0x68, 0xf0, 0x2d, 0x88, 0x8b, 0xfc, 0x91, 0xfe, 0xb6, 0x45, 0xd9
    ];

    // uniswap deployer address, from https://blog.uniswap.org/uniswap-v4-address-mining-challenge
    const deployer: [u8; 20] = [
        0x48, 0xE5, 0x16, 0xB3, 0x4A, 0x12, 0x74, 0xf4, 0x94, 0x57,
        0xb9, 0xC6, 0x18, 0x20, 0x97, 0x79, 0x6D, 0x04, 0x98, 0xCb
    ];

    let mut hasher = Keccak256::new();
    hasher.update(&[0xFF]);
    hasher.update(&deployer);
    hasher.update(&salt);
    hasher.update(&bytecode_hash);

    let result = hasher.finalize();

    // Take the last 20 bytes of the hash as the address
    let mut address = [0u8; 20];
    address.copy_from_slice(&result[12..]);

    address
}

#[wasm_bindgen]
pub struct AddressScore {
    address: String,
    score: u64,
    salt: String,
}


#[wasm_bindgen]
pub fn loop_hash(sender: Vec<u8>, times: u16, bestScore: u64) -> AddressScore {
    let caller: [u8; 20] = [
        0x2c, 0x8b, 0x14, 0xa2, 0x70, 0xeb, 0x08, 0x0c, 0x26, 0x62, 
        0xa1 ,0x29 ,0x36 ,0xbb ,0x6b ,0x2b ,0xab ,0xf1 ,0x5b ,0xf8
    ];

    // first 20 bytes of the salt are the sender address
    let mut salt = [0u8; 32];
    salt[..20].copy_from_slice(&sender);      

    let mut address = [0u8; 20];
    let mut score = 0;
    for _ in 0..times {
        address = compute_address(&salt);
        score = score::score(&address);

        if score > bestScore {
            return AddressScore {
                address: hex::encode(address),
                salt: hex::encode(salt),
                score: score::score(&address),
            }
        }
    }

    AddressScore {
        address: hex::encode(address),
        salt: hex::encode(salt),
        score: score::score(&address),
    }
}
