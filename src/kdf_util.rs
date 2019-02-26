use argon2::{Config, ThreadMode, Variant};
use error::{Error, Result};


pub fn derive_key(password: &[u8], salt: &[u8], hash_len: u32) -> Result<Vec<u8>> {
    let mut config = Config::default();
    config.variant = Variant::Argon2id;
    config.mem_cost = 65536;
    config.time_cost = 3;
    config.hash_length = hash_len;
    config.lanes = 4;
    config.thread_mode = ThreadMode::from_threads(4);

    // encoded hash
    argon2::hash_raw(password, salt, &config).map_err(|_| Error::InvalidDataFile)
}
