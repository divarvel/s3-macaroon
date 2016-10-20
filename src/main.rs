extern crate iron;
extern crate rusoto;
extern crate avro;
extern crate router;
extern crate ring;
extern crate rustc_serialize;

use std::io::Read;
use iron::prelude::*;
use iron::status;
use iron::request::Body;
use router::Router;
use rustc_serialize::base64::{FromBase64,ToBase64,URL_SAFE};

mod credentials;
mod crypto;
mod key;

use credentials::{Credentials, S3SignatureVersion};

fn main() {
    // Ensure AES_KEY is loaded when starting
    let _ = key::get_key_from_env().unwrap();
    let mut router = Router::new();

    router.get("/", hello, "index");
    router.get("/encode", test_encode, "test-encode");
    router.post("/decode", test_decode, "test-decode");
    Iron::new(router).http("0.0.0.0:8081").unwrap();
}

fn test_encode(_: &mut Request) -> IronResult<Response> {
    let key_bytes = key::get_key_from_env().unwrap();
    let creds = Credentials {
        access_key: "Test".into(),
        access_secret: "secret".into(),
        base: "cellar.services.clever-cloud.com".into(),
        signature_version: S3SignatureVersion::V2,
        macaroon_secret: "toto".into(),
    };
    match encode_then_encrypt(creds, &key_bytes) {
        Ok(result) => Ok(Response::with((status::Ok, result))),
        _          => Ok(Response::with((status::InternalServerError, "")))
    }
}

fn test_decode(r: &mut Request) -> IronResult<Response> {
    let key_bytes = key::get_key_from_env().unwrap();
    if let Ok(result) = decrypt_then_decode(&mut r.body, &key_bytes) {
        Ok(Response::with((status::Ok, format!("{:?}", result))))
    } else {
        Ok(Response::with((status::BadRequest, "")))
    }
}

fn encode_then_encrypt(creds: Credentials, key_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let encoded = try!(credentials::encode_credentials(creds));
    let encrypted = try!(crypto::encrypt(&encoded, key_bytes).map_err(|_| "Couldn't encrypt"));
    let encrypted_hex = encrypted.to_base64(URL_SAFE).into_bytes();

    Ok(encrypted_hex)
}

fn decrypt_then_decode<'a,'b>(body: &mut Body, key_bytes: &[u8]) -> Result<Credentials, String> {
    let mut bytes = Vec::new();
    let _ = try!(body.read_to_end(&mut bytes).map_err(|_| "Couldn't read"));
    let encrypted = try!(bytes.from_base64().map_err(|_| "Couldn't decode"));
    let encoded = try!(crypto::decrypt(&encrypted, key_bytes).map_err(|_| "ToDo"));
    let creds = try!(credentials::decode_credentials(&encoded));

    Ok(creds)
}



fn hello(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello World from rust!")))
}

#[allow(dead_code)]
fn read_param<'a>(r: &'a Request, name: &str) -> Option<&'a str> {
    r.extensions.get::<Router>().and_then(|r| r.find(name))
}


