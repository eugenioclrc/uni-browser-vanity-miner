use wasm_bindgen::prelude::*;

use sha3::{Keccak256, Digest};
use hex;
use rand::Rng;

/// Returns the nibble at a given index in an address
/// 
/// # Arguments
/// * `input` - A 20-byte address to get the nibble from
/// * `nibble_index` - The index of the nibble to retrieve
/// 
/// # Returns
/// * `u8` - The nibble value at the specified index
fn get_nibble(input: &[u8; 20], nibble_index: usize) -> u8 {
    // Get the byte corresponding to the nibble index
    let curr_byte = input[nibble_index / 2];
    
    if nibble_index % 2 == 0 {
        // Return the higher nibble of the byte
        curr_byte >> 4
    } else {
        // Return the lower nibble of the byte
        curr_byte & 0x0F
    }
}

/// Returns the number of leading nibbles in an address that match a given value
/// 
/// # Arguments
/// * `input` - A 20-byte address to analyze
/// * `start_index` - The starting nibble index
/// * `comparison` - The nibble value to compare against
/// 
/// # Returns
/// * `usize` - The count of leading matching nibbles
fn get_leading_nibble_count(input: &[u8; 20], start_index: usize, comparison: u8) -> usize {
    let mut count = 0;
    for i in start_index..(input.len() * 2) {
        let current_nibble = get_nibble(input, i);
        if current_nibble != comparison {
            break;
        }
        count += 1;
    }
    count
}

/// Scores an address based on its vanity
/// 
/// # Arguments
/// * `input` - A 20-byte address to score
/// 
/// # Returns
/// * `usize` - The calculated vanity score
pub fn score(input: &[u8; 20]) -> usize {
    let mut calculated_score = 0;

    // 10 points per leading zero nibble
    let leading_zero_count = get_leading_nibble_count(input, 0, 0);
    calculated_score += (leading_zero_count * 10) as usize;

    // Special handling for 4s immediately after leading 0s
    let leading_four_count = get_leading_nibble_count(input, leading_zero_count, 4);

    if leading_four_count == 0 {
        return 0;
    } else if leading_four_count == 4 {
        calculated_score += 60;
    } else if leading_four_count > 4 {
        calculated_score += 40;
    }

    // 1 extra point for any 4 nibbles
    for i in 0..(input.len() * 2) {
        if get_nibble(input, i) == 4 {
            calculated_score += 1;
        }
    }

    // If the last 4 nibbles are 4s, add 20 points
    if input[18] == 0x44 && input[19] == 0x44 {
        calculated_score += 20;
    }

    calculated_score
}

fn compute_address(salt: &[u8; 32]) -> [u8; 20] {
    // uniswap bytecode hash, from https://blog.uniswap.org/uniswap-v4-address-mining-challenge
    const BYTECODE_HASH: [u8; 32] = [
        0x94, 0xd1, 0x14, 0x29, 0x6a, 0x5a, 0xf8, 0x5c, 0x1f, 0xd2, 0xdc, 0x03, 0x9c, 0xda, 0xa3, 0x2f, 
        0x1e, 0xd4, 0xb0, 0xfe, 0x08, 0x68, 0xf0, 0x2d, 0x88, 0x8b, 0xfc, 0x91, 0xfe, 0xb6, 0x45, 0xd9
    ];

    // uniswap deployer address, from https://blog.uniswap.org/uniswap-v4-address-mining-challenge
    const DEPLOYER: [u8; 20] = [
        0x48, 0xE5, 0x16, 0xB3, 0x4A, 0x12, 0x74, 0xf4, 0x94, 0x57,
        0xb9, 0xC6, 0x18, 0x20, 0x97, 0x79, 0x6D, 0x04, 0x98, 0xCb
    ];

    let mut hasher = Keccak256::new();
    hasher.update(&[0xFF]);
    hasher.update(&DEPLOYER);
    hasher.update(salt);
    hasher.update(&BYTECODE_HASH);

    let result = hasher.finalize();

    // Take the last 20 bytes of the hash as the address
    let mut address = [0u8; 20];
    address.copy_from_slice(&result[12..32]);

    address
}

#[wasm_bindgen]
pub struct AddressScore {
    address: String,
    score: usize,
    salt: String,
}

#[wasm_bindgen]
impl AddressScore {
    #[wasm_bindgen(constructor)]
    pub fn new(address: String, score: usize, salt: String) -> AddressScore {
        AddressScore { address, score, salt }
    }

    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn score(&self) -> usize {
        self.score
    }

    #[wasm_bindgen(getter)]
    pub fn salt(&self) -> String {
        self.salt.clone()
    }
}

#[wasm_bindgen]
pub fn loop_hash(_sender: String, times: usize, best_score: usize) ->  AddressScore {
    // first 20 bytes of the salt are the sender address
    // let sender = hex::decode("6CA6d1e2D5347Bfab1d91e883F1915560e09129D").unwrap();
    let sender_trimmed = if _sender.starts_with("0x") {
        &_sender[2..]
    } else {
        &_sender
    };

    // Decodificar la dirección
    let sender = hex::decode(sender_trimmed).unwrap();
    // Verificar la longitud de la dirección
    if sender.len() != 20 {
        panic!("Wrong address length");
    }
    let mut salt = [0u8; 32];
    salt[..20].copy_from_slice(&sender);

    let mut best_address = String::new();
    let mut best_salt = String::new();
    let mut current_best_score = best_score;

    let mut rng = rand::thread_rng();

    // Set the last 12 bytes of the salt to random values
    for i in 0..12 {
        salt[31 - i] = rng.gen();
    }

    for i in 0..times {
        // Update salt to have a unique value for each iteration
        salt[31-(i%12)] = rng.gen();

        let address = compute_address(&salt);
        let _score = score(&address);

        if _score > current_best_score {
            current_best_score = _score;
            best_address = hex::encode(address);
            best_salt = hex::encode(salt);
            break;
        }
    }
    //best_salt

    AddressScore::new(best_address, current_best_score, best_salt)
}
