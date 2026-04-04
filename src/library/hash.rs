use petname::Generator;
use rand::SeedableRng;
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
    let hash = Sha256::digest(input.as_bytes());
    let mut rng = rand::rngs::StdRng::from_seed(hash.into());

    let mut buf = String::new();
    petname::Petnames::default()
        .generate_into(&mut buf, &mut rng, 2, "-");
    buf
}
