use hex;
use sodiumoxide::crypto::{
    hash::sha256,
    sign::ed25519,
    sign::ed25519::{PublicKey, SecretKey, Seed, Signature},
};

/// Initializes the sodium library and automatically selects faster versions
/// of the primitives, if possible.
pub fn init() {
    if sodiumoxide::init().is_err() {
        panic!("Cryptographic library hasn't initialized.");
    }
}

/// Signs a slice of bytes using the signer's secret key and returns the
/// resulting `Signature`.
pub fn sign(data: &[u8], secret_key: &SecretKey) -> Signature {
    ed25519::sign_detached(data, secret_key)
}

/// Computes a secret key and a corresponding public key from a `Seed`.
pub fn gen_keypair_from_seed(seed: &Seed) -> (PublicKey, SecretKey) {
    ed25519::keypair_from_seed(seed)
}

/// Generates a secret key and a corresponding public key using a cryptographically secure
/// pseudo-random number generator.
pub fn gen_keypair() -> (PublicKey, SecretKey) {
    ed25519::gen_keypair()
}

/// Verifies that `data` is signed with a secret key corresponding to the
/// given public key.
pub fn verify(sig: &Signature, data: &[u8], pub_key: &PublicKey) -> bool {
    ed25519::verify_detached(sig, data, pub_key)
}

/// Returns a hex representation of binary data.
pub fn to_hex_pk(pk: &PublicKey) -> String {
    hex::encode(&pk[..])
}

/// Returns a hex representation of binary data.
pub fn to_hex_sk(pk: &SecretKey) -> String {
    hex::encode(&pk[..])
}

/*
/// Calculates hash of a bytes slice.
pub fn hash(data: &[u8]) -> Hash {
    sha256::hash(data)
}

pub fn hash(data: &[u8]) -> Hash {
    let dig = crypto_impl::hash(data);
    Hash(dig)
}
*/
