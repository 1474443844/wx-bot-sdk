use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, block_padding::Pkcs7};

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

pub fn encrypt_aes_ecb(plaintext: &[u8], key: &[u8]) -> crate::Result<Vec<u8>> {
    let cipher = Aes128EcbEnc::new_from_slice(key)?;
    Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext))
}

pub fn decrypt_aes_ecb(ciphertext: &[u8], key: &[u8]) -> crate::Result<Vec<u8>> {
    let cipher = Aes128EcbDec::new_from_slice(key)?;
    cipher
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|_| "AES-ECB decrypt padding error".into())
}

pub fn aes_ecb_padded_size(plaintext_size: usize) -> usize {
    plaintext_size + (16 - (plaintext_size % 16))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let key = [7_u8; 16];
        let plaintext = b"hello world";
        let enc = encrypt_aes_ecb(plaintext, &key).unwrap();
        assert_eq!(enc.len(), aes_ecb_padded_size(plaintext.len()));
        let dec = decrypt_aes_ecb(&enc, &key).unwrap();
        assert_eq!(dec, plaintext);
    }
}
