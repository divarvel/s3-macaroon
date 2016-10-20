use std::rc::Rc;
use avro::{decode, encode, EnumSchema, EnumSymbol, Field, RecordSchema, Schema, Value};
use std::io::BufReader;

#[derive(Debug)]
pub enum S3SignatureVersion {
    V2,
    V4,
}

pub fn s3_signature_version_schema<'a>() -> EnumSchema<'a> {
    EnumSchema {
        name: "S3SignatureVersion".into(),
        doc: None,
        properties: vec![],
        symbols: vec![
            EnumSymbol { name: "V2".into(), doc: None },
            EnumSymbol { name: "V4".into(), doc: None },
        ],
    }
}


impl<'a, 'b> Into<Value<'a, 'b>> for S3SignatureVersion {
    fn into(self) -> Value<'a, 'b> {
        let schema = s3_signature_version_schema();
        Value::Enum(Rc::new(schema),
                    match self {
                        S3SignatureVersion::V2 => 0,
                        S3SignatureVersion::V4 => 1,
                    })
    }
}

pub fn value_to_signature_version(v: &Value) -> Result<S3SignatureVersion, String> {
    match v {
        &Value::Enum(_, 0) => Ok(S3SignatureVersion::V2),
        &Value::Enum(_, 1) => Ok(S3SignatureVersion::V4),
        _ => Err("invalid signature version".into())
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub access_key: String,
    pub access_secret: String,
    pub base: String,
    pub signature_version: S3SignatureVersion,
    pub macaroon_secret: String,
}

pub fn credentials_schema<'a>() -> RecordSchema<'a> {
    RecordSchema::new(
        "S3SignatureVersion".into(),
        None,
        vec![],
        vec![
            Field { name: "access_key".into(), doc: None, properties: vec![], ty: Schema::String },
            Field { name: "access_secret".into(), doc: None, properties: vec![], ty: Schema::String },
            Field { name: "base".into(), doc: None, properties: vec![], ty: Schema::String },
            Field { name: "signature_version".into(), doc: None, properties: vec![], ty: Schema::Enum(Rc::new(s3_signature_version_schema())) },
            Field { name: "macaroon_secret".into(), doc: None, properties: vec![], ty: Schema::String },
        ]
    )
}

impl<'a, 'b> Into<Value<'a, 'b>> for Credentials {
    fn into(self) -> Value<'a, 'b> {
        let schema = credentials_schema();
        Value::Record(Rc::new(schema),
                      vec![
            Value::String(self.access_key.into()),
            Value::String(self.access_secret.into()),
            Value::String(self.base.into()),
            self.signature_version.into(),
            Value::String(self.macaroon_secret.into()),
        ])
    }
}

pub fn value_to_creds(v: Value) -> Result<Credentials, String> {
    match v {
        Value::Record(_, ref fields) => {
            if fields.len() == 5 {
                let ref key = fields[0];
                let ref secret = fields[1];
                let ref base = fields[2];
                let ref sig_version = fields[3];
                let ref macaroon_secret = fields[4];

                match (key, secret, base, macaroon_secret) {
                    (
                        &Value::String(ref k),
                        &Value::String(ref s),
                        &Value::String(ref b),
                        &Value::String(ref ms)
                    ) => {
                        match value_to_signature_version(sig_version) {
                            Ok(sv) => {
                                Ok(
                                    Credentials {
                                        access_key: (*k.clone()).into(),
                                        access_secret: (*s.clone()).into(),
                                        base: (*b.clone()).into(),
                                        signature_version: sv,
                                        macaroon_secret: (*ms.clone()).into()
                                    }
                                )
                            },
                            Err(e) => Err(e)
                        }

                    },
                    _ => Err("Invalid record".into())
                }
            } else {
                Err("Invalid record".into())
            }
        },
        _ => Err("Not a record".into())
    }
}

pub fn encode_credentials(creds: Credentials) -> Result<Vec<u8>,String> {
    let mut output = Vec::new();
    let schema = Schema::Record(Rc::new(credentials_schema()));
    let _ = try!(encode(&mut output, &schema, &creds.into()).map_err(|_| "Couldn't encode credentials"));

    Ok(output)
}

pub fn decode_credentials(bytes: &[u8]) -> Result<Credentials,String> {
    let mut reader = BufReader::new(bytes);
    let schema = Schema::Record(Rc::new(credentials_schema()));

    let value = try!(decode(&mut reader, &schema).map_err(|_| "TODO"));
    let creds = try!(value_to_creds(value));
    Ok(creds)
}
