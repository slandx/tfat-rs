use data_encoding::BASE32_NOPAD;
use error::{Error, Result};
use ring::{digest, hmac};
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Debug)]
pub struct TOTP {
    key: Vec<u8>,
    output_len: usize,
    output_base: Vec<u8>,
}

impl TOTP {
    pub fn new(
        key: &str,
        output_len: Option<usize>,
    ) -> Result<TOTP> {
        let decoded_key = BASE32_NOPAD
            .decode(key.as_bytes())
            .map_err(|err| Error::KeyDecode {
                key: key.to_owned(),
                cause: Box::new(err),
            })?;
        let output_len = match output_len {
            Some(len) => len,
            None => 6,
        };
        let otp = TOTP {
            key: decoded_key,
            output_len,
            output_base: "0123456789".to_owned().into_bytes(),
        };
        Ok(otp)
    }

    // Generate a code as defined in [RFC4226](https://tools.ietf.org/html/rfc4226)
    pub fn generate(&self) -> (String, u32) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64;
        let (counter, remain) = (timestamp / 30, 30 - timestamp % 30);

        let message: [u8; 8] = [
            ((counter >> 56) & 0xff) as u8,
            ((counter >> 48) & 0xff) as u8,
            ((counter >> 40) & 0xff) as u8,
            ((counter >> 32) & 0xff) as u8,
            ((counter >> 24) & 0xff) as u8,
            ((counter >> 16) & 0xff) as u8,
            ((counter >> 8) & 0xff) as u8,
            (counter & 0xff) as u8,
        ];
        let signing_key = hmac::SigningKey::new(&digest::SHA1, &self.key);
        let digest = hmac::sign(&signing_key, &message);
        (self.encode_digest(digest.as_ref()), remain as u32)
    }

    fn encode_digest(&self, digest: &[u8]) -> String {
        let offset = (*digest.last().unwrap() & 0xf) as usize;
        let snum: u32 = ((u32::from(digest[offset]) & 0x7f) << 24)
            | ((u32::from(digest[offset + 1]) & 0xff) << 16)
            | ((u32::from(digest[offset + 2]) & 0xff) << 8)
            | (u32::from(digest[offset + 3]) & 0xff);
        let base = self.output_base.len() as u32;
        let hotp_code = snum % base.pow(self.output_len as u32);
        let code = format!("{:0width$}", hotp_code, width = self.output_len);
        code
    }
}
