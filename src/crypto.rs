use std::{fmt::Write, num::ParseIntError};

//snatched from https://www.reddit.com/r/rust/comments/3qvmma/encrypting_file_with_asymmetric_encryption/
use crypto::aead::{AeadDecryptor, AeadEncryptor};
use crypto::chacha20poly1305::ChaCha20Poly1305;
use crypto::curve25519::{curve25519, curve25519_base};
use rand::{OsRng, Rng};

pub enum EncryptError {
    RngInitializationFailed,
}

pub fn encrypt(public_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, EncryptError> {
    let mut rng = OsRng::new().map_err(|_| EncryptError::RngInitializationFailed)?;

    let mut ephemeral_secret_key = [0u8; 32];
    rng.fill_bytes(&mut ephemeral_secret_key[..]);

    let ephemeral_public_key: [u8; 32] = curve25519_base(&ephemeral_secret_key[..]);
    let symmetric_key = curve25519(&ephemeral_secret_key[..], &public_key[..]);

    let mut c = ChaCha20Poly1305::new(&symmetric_key, &[0u8; 8][..], &[]);

    let mut output = vec![0; 32 + 16 + message.len()];
    let mut tag = [0u8; 16];
    c.encrypt(message, &mut output[32 + 16..], &mut tag[..]);

    for (dest, src) in (&mut output[0..32])
        .iter_mut()
        .zip(ephemeral_public_key.iter())
    {
        *dest = *src;
    }

    for (dest, src) in (&mut output[32..48]).iter_mut().zip(tag.iter()) {
        *dest = *src;
    }

    Ok(output)
}

pub fn encrypt_string(public_key: &[u8; 32], message: String) -> String {
    // converting our input message to a vector of bytes
    let message_bytes = message.as_bytes().to_vec();
    // here we use &* to convert the vector to [u8]
    let result = encrypt(&public_key, &*message_bytes).ok().unwrap();
    // println!("Encrypted Bytes: {:?}", result);
    //we format the bytes as strings
    encode_hex(&result)
}

pub fn decrypt_string(secret_key: &[u8; 32], message: String) -> String {
    // converting our input message to a vector of bytes
    let message_bytes = decode_hex(&message).unwrap();
    // println!("Encrypted Bytes: {:?}", message_bytes);
    //
    let result = decrypt(&secret_key, &message_bytes).ok().unwrap();
    //we format the bytes as strings
    String::from_utf8(result).unwrap()
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub enum DecryptError {
    Malformed,
    Invalid,
}

pub fn decrypt(secret_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, DecryptError> {
    if message.len() < 48 {
        return Err(DecryptError::Malformed);
    }

    let ephemeral_public_key = &message[0..32];
    let tag = &message[32..48];
    let ciphertext = &message[48..];

    let mut plaintext = vec![0; ciphertext.len()];
    let symmetric_key = curve25519(secret_key, ephemeral_public_key);

    let mut decrypter = ChaCha20Poly1305::new(&symmetric_key[..], &[0u8; 8][..], &[]);
    if !decrypter.decrypt(ciphertext, &mut plaintext[..], tag) {
        return Err(DecryptError::Invalid);
    }

    Ok(plaintext)
}

pub fn generate_key_pair() -> ([u8; 32], [u8; 32]) {
    let mut secret_key = [0u8; 32];
    OsRng::new().unwrap().fill_bytes(&mut secret_key[..]);
    let public_key = curve25519_base(&secret_key[..]);
    (public_key, secret_key)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn it_works() {
        let mut secret_key = [0u8; 32];
        OsRng::new().unwrap().fill_bytes(&mut secret_key[..]);

        let public_key = curve25519_base(&secret_key[..]);

        let encrypted_message = encrypt(&public_key, b"Just a test").ok().unwrap();

        let decrypted_message = decrypt(&secret_key, &encrypted_message[..]).ok().unwrap();

        assert_eq!(decrypted_message, b"Just a test".to_vec());

        {
            // Corrupt the ephemeral public key
            let mut corrupt_1 = encrypted_message.clone();
            corrupt_1[3] ^= 1;
            assert!(decrypt(&secret_key, &corrupt_1[..]).is_err());
        }

        {
            // Corrupt the tag
            let mut corrupt_2 = encrypted_message.clone();
            corrupt_2[35] ^= 1;
            assert!(decrypt(&secret_key, &corrupt_2[..]).is_err());
        }

        {
            // Corrupt the message
            let mut corrupt_3 = encrypted_message.clone();
            corrupt_3[50] ^= 1;
            assert!(decrypt(&secret_key, &corrupt_3[..]).is_err());
        }
    }
    #[test]
    fn high_level_test() {
        let (public, private) = generate_key_pair();
        let test_string = "Testing encryption";
        // println!("Original: {}", test_string);
        let encrypted = encrypt_string(&public, test_string.to_string());
        // println!("Encrypted: {}", encrypted);
        let decrypted = decrypt_string(&private, encrypted);
        // println!("Decrypted: {}", decrypted);
        assert_eq!(test_string, decrypted);
    }
}
