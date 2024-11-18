
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
/// * `u64` - The calculated vanity score
pub fn score(input: &[u8; 20]) -> u64 {
    let mut calculated_score = 0;

    // 10 points per leading zero nibble
    let leading_zero_count = get_leading_nibble_count(input, 0, 0);
    calculated_score += (leading_zero_count * 10) as u64;

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