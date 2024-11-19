mod lib;

fn main() {
    let sender: String = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string();


    let results = lib::loop_hash(sender, 1000, 0);
    println!("Address: {}", &results.address());
    println!("Score: {}", results.score());
    println!("Salt: {}", &results.salt());
}