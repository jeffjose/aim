use petname::Generator;
use sha2::{Digest, Sha256};

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn sha256_short(input: &str) -> String {
    sha256(input)[..12].to_string()
}

pub fn petname(input: &str) -> String {
    let mut rng: rand_chacha::ChaCha8Rng = rand_seeder::Seeder::from(input).into_rng();

    petname::Petnames::default()
        .generate(&mut rng, 2, "-")
        .expect("Failed to create a petname")
}
