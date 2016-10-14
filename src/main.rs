extern crate iron;
extern crate avro;

use iron::prelude::*;
use iron::status;

fn main() {
    Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World!")))
    }).http("localhost:8080").unwrap();
}

pub struct HttpFront {
    pub app_id: String,
    pub hostname: String,
    pub path_begin: String,
}
fn test() {
    use std::rc::Rc;
    use avro::{Field, Schema, RecordSchema, Value, decode};

    let fields = vec![
        Field { name: "year".into(), doc: None, properties: vec![], ty: Schema::Int },
        Field { name: "color".into(), doc: None, properties: vec![], ty: Schema::String },
        Field { name: "running".into(), doc: None, properties: vec![], ty: Schema::Boolean },
    ];
    let schema = Rc::new(RecordSchema::new("Car".into(), None, vec![], fields));
    if let Value::Record(_, rec_data) =
    decode(&mut &b"\xAE\x1F\x06\x52\x65\x64\x01"[..], &Schema::Record(schema)).unwrap() {
        assert_eq!(rec_data[0], Value::Int(2007));
        assert_eq!(rec_data[1], Value::String("Red".into()));
        assert_eq!(rec_data[2], Value::Boolean(true));
        println!("Success!");
    } else {
        panic!("wrong type");
    }

}

