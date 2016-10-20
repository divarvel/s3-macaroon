use ring::rand::SystemRandom;
use ring::aead::*;
use ring::error::Unspecified;

pub fn encrypt<'a>(data: &'a [u8], key_bytes: &[u8]) -> Result<Vec<u8>, Unspecified> {
    let mut nonce = [0u8; 12];
    let generator = SystemRandom::new();
    let _ = try!(generator.fill(&mut nonce[..]));

    let mut output = Vec::new();
    let key = try!(SealingKey::new(&AES_256_GCM, key_bytes));
    let max_overhead_len = key.algorithm().max_overhead_len();

    output.extend_from_slice(&nonce);
    output.extend_from_slice(&data);
    output.resize(
        nonce.len() + data.len() + max_overhead_len,
        0u8);
    
    let _ = try!(seal_in_place(&key,
                               &nonce,
                               &mut output[12..],
                               key.algorithm().max_overhead_len(),
                               b""));

    Ok(output)
}

pub fn decrypt<'a>(data: &'a [u8], key_bytes: &[u8]) -> Result<Vec<u8>, Unspecified> {
    let key = try!(OpeningKey::new(&AES_256_GCM, key_bytes));
    let nonce = &data[0..key.algorithm().nonce_len()];

    let mut output = Vec::new();
    output.extend_from_slice(&data[key.algorithm().nonce_len()..]);
    let out_len = try!(open_in_place(&key, nonce, 0, &mut output, b""));

    output.truncate(out_len);
    Ok(output)
}

fn encrypt_and_decrypt<'a>(data: &'a [u8], key: &[u8]) -> Result<Vec<u8>, Unspecified> {
    let encrypted = try!(encrypt(data, key));
    let decrypted = try!(decrypt(&encrypted, key));
    Ok(decrypted)
}

#[test]
pub fn test() {
    let generator = SystemRandom::new();
    let data = b"abcd";
    let mut key = [0u8; 32];
    let _ = generator.fill(&mut key).unwrap();


    if let Ok(res) = encrypt_and_decrypt(data, &key) {
        assert_eq!(&data[..], &res[..]);
    } else {
        assert_eq!(true, false);
    }
}
