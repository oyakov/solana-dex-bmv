use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let password = if args.len() > 1 { &args[1] } else { "admin123" };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    println!("HASH_START{}HASH_END", password_hash);
}
