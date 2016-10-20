use std::env;
use rustc_serialize::hex::FromHex;

pub fn get_key_from_env() -> Result<Vec<u8>, String> {
    let string = try!(env::var("AES_KEY").map_err(|_| "Missing AES_KEY env var"));
    let bytes = try!(string.from_hex().map_err(|_| "Invalid key"));

    if bytes.len() != 32 {
        Err("Invalid key length".into())
    } else {
        Ok(bytes)
    }
}