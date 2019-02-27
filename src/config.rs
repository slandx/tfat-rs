use dirs;
use error::{Error, Result};
use kdf_util;
use ring::aead;
use ring::rand::{SecureRandom, SystemRandom};
use rpassword;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PwdType {
    UserPwd = 1,
    DefaultPwd = 2,
}

impl PwdType {
    fn from_u8(value: u8) -> PwdType {
        match value {
            1 => PwdType::UserPwd,
            2 => PwdType::DefaultPwd,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub empty: bool,
    pub pwd: Vec<u8>,
    pub pwd_type: PwdType,
    pub accounts: HashMap<String, String>,
}

const APP_DIR: &str = ".tfat";
const CONFIG_FILE_NAME: &str = "tfat.dat";

/// +-------+------+-----------+
/// | nonce | type | encrypted |
/// +-------+------+-----------+
/// | 12B   | 1B   | ...       |
/// +-------+------+-----------+

pub fn read_from_file() -> Result<Config> {
    let app_dir = Path::new(&dirs::home_dir().ok_or(Error::HomeDirNotFound)?).join(APP_DIR);
    let file_path = get_file(&app_dir, CONFIG_FILE_NAME)?;
    let cfg_buf = fs::read(file_path)?;

    let encrypted_pos = 1 + aead::NONCE_LEN;
    if cfg_buf.len() == 0 {
        return Ok(Config {
            empty: true,
            pwd: vec![],
            pwd_type: PwdType::DefaultPwd,
            accounts: HashMap::new(),
        });
    } else if cfg_buf.len() < encrypted_pos {
        return Err(Error::InvalidDataFile);
    }

    let mut cfg = Config {
        empty: false,
        pwd: vec![],
        pwd_type: PwdType::from_u8(cfg_buf[aead::NONCE_LEN]),
        accounts: HashMap::new(),
    };

    let mut nonce = [0u8; aead::NONCE_LEN];
    nonce.copy_from_slice(&cfg_buf[..aead::NONCE_LEN]);

    let mut read_buf = vec![];
    cfg.pwd.extend_from_slice(match cfg.pwd_type {
        PwdType::UserPwd => {
            read_buf.extend_from_slice(rpassword::prompt_password_stdout("Password: ").unwrap().as_bytes());
            read_buf.as_slice()
        }
        PwdType::DefaultPwd => {
            &nonce
        }
        _=> {
            return Err(Error::InvalidDataFile);
        }
    });
    // create AES-GCM open key derived with argon2
    let derived_key = kdf_util::derive_key(cfg.pwd.as_slice(), &nonce, aead::AES_256_GCM.key_len() as u32)?;
    let open_key = aead::OpeningKey::new(&aead::AES_256_GCM, derived_key.as_slice()).unwrap();

    let mut in_out = vec![0; cfg_buf.len() - encrypted_pos];
    in_out.copy_from_slice(&cfg_buf[encrypted_pos..]);

    let decrypted_data = aead::open_in_place(&open_key, aead::Nonce::assume_unique_for_key(nonce),
                                             aead::Aad::empty(), 0, &mut in_out).map_err(|_| Error::WrongPassword)?;

    cfg.accounts = toml::from_slice(decrypted_data)?;
    Ok(cfg)
}

pub fn save_to_file(cfg: &Config) -> Result<bool> {
    let app_dir = Path::new(&dirs::home_dir().ok_or(Error::HomeDirNotFound)?).join(APP_DIR);
    let file_path = get_file(&app_dir, CONFIG_FILE_NAME)?;

    let accounts_str = toml::to_string(&cfg.accounts)?;

    // Ring uses the same input variable as output
    let mut in_out = accounts_str.into_bytes();

    // The input/output variable need some space for a suffix
    for _ in 0..aead::AES_256_GCM.tag_len() {
        in_out.push(0);
    }

    // Fill nonce with random data
    let mut nonce = [0u8; aead::NONCE_LEN];
    let rand = SystemRandom::new();
    rand.fill(&mut nonce).unwrap();

    let mut write_buf = vec![];
    write_buf.extend_from_slice(&nonce);
    write_buf.push(cfg.pwd_type.clone() as u8);

    // check password
    let password = match cfg.pwd_type {
        PwdType::UserPwd => cfg.pwd.as_slice(),
        PwdType::DefaultPwd => &nonce
    };
    // create AES-GCM seal key derived with argon2
    let derived_key = kdf_util::derive_key(password, &nonce, aead::AES_256_GCM.key_len() as u32)?;
    let seal_key = aead::SealingKey::new(&aead::AES_256_GCM, derived_key.as_slice()).unwrap();
    aead::seal_in_place(&seal_key, aead::Nonce::assume_unique_for_key(nonce),
                        aead::Aad::empty(), &mut in_out, aead::AES_256_GCM.tag_len()).unwrap();

    write_buf.extend_from_slice(&in_out);
    fs::write(file_path, write_buf)?;

    Ok(true)
}

fn get_file(dir: &PathBuf, file_name: &str) -> Result<PathBuf> {
    fs::create_dir_all(&dir)?;
    let file_path = dir.join(file_name);
    if !file_path.is_file() {
        let _ = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path);
    }
    Ok(file_path)
}

pub fn init_pwd(cfg: &mut Config) -> Result<bool> {
    let mut pwd_buf = vec![];
    let mut confirm_pwd_buf = vec![];
    let mut retry_times = 2;
    loop {
        pwd_buf.extend_from_slice(rpassword::prompt_password_stdout("New password: ").unwrap().as_bytes());
        confirm_pwd_buf.extend_from_slice(rpassword::prompt_password_stdout("Confirm password: ").unwrap().as_bytes());
        if pwd_buf.iter().eq(confirm_pwd_buf.iter()) {
            break;
        } else if retry_times > 0 {
            retry_times -= 1;
            println!("Different password, try again!");
            pwd_buf.clear();
            confirm_pwd_buf.clear();
        } else {
            return Err(Error::WrongPassword);
        }
    }

    if pwd_buf.len() == 0 {
        cfg.pwd_type = PwdType::DefaultPwd;
    } else {
        cfg.pwd = pwd_buf;
        cfg.pwd_type = PwdType::UserPwd;
    }
    Ok(true)
}
