use solana_sdk::signature::Keypair;
fn main() {
    let kp1 = Keypair::new();
    let kp2 = Keypair::new();
    println!("KP1={}", kp1.to_base58_string());
    println!("KP2={}", kp2.to_base58_string());
}
