#![cfg(feature = "json")]
use crate::Config;
use cfgfifo::*;
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::{read_to_string, Seek, Write};
use tempfile::{tempfile, Builder};

static JSON: &str = indoc! {r#"
{
  "primitives": {
    "integer": 42,
    "float": 1.618,
    "boolean": true,
    "text": "This is test text.\nThis is a new line.\n\tThis is an indented line.\nThis is a snowman with a goat: ☃🐐.",
    "none": null,
    "some": 17,
    "list": [
      1,
      2,
      6,
      15,
      36
    ],
    "dict": {
      "hello": "goodbye",
      "strange": "charmed",
      "up": "down"
    }
  },
  "enums": {
    "color": "green",
    "msg": {
      "type": "Response",
      "id": 60069,
      "value": "Foobar"
    }
  },
  "people": [
    {
      "id": 1,
      "given_name": "Alice",
      "family_name": "Alison"
    },
    {
      "id": 2,
      "given_name": "Bob",
      "family_name": "Bobson"
    },
    {
      "id": 3,
      "given_name": "Charlie",
      "family_name": "McCharles"
    }
  ]
}"#};

#[test]
fn load_from_str() {
    let r = Format::Json.load_from_str::<Config>(JSON);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_string() {
    let r = Format::Json.dump_to_string(&Config::get());
    assert_eq!(r.unwrap(), JSON);
}

#[test]
fn load_from_reader() {
    let mut file = tempfile().unwrap();
    writeln!(file, "{JSON}").unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = Format::Json.load_from_reader::<_, Config>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_writer() {
    let mut file = tempfile().unwrap();
    let r = Format::Json.dump_to_writer(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, format!("{JSON}\n"));
    assert!(s.ends_with("}\n"));
}

#[test]
fn load_from_file() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
    writeln!(file, "{JSON}").unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = load::<Config, _>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_file() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
    let r = dump(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, format!("{JSON}\n"));
    assert!(s.ends_with("}\n"));
}

#[test]
fn fallback_load() {
    let mut file = Builder::new().suffix(".unk").tempfile().unwrap();
    writeln!(file, "{JSON}").unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let cfg = Cfgfifo::new().fallback(Some(Format::Json));
    let r = cfg.load::<Config, _>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn fallback_dump() {
    let mut file = Builder::new().suffix(".unk").tempfile().unwrap();
    let cfg = Cfgfifo::new().fallback(Some(Format::Json));
    let r = cfg.dump(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, format!("{JSON}\n"));
    assert!(s.ends_with("}\n"));
}

#[test]
fn deserialize_error() {
    let s = indoc! {r#"
       {
         "primitives": {
           "integer": 3.14
         },
         "enums": {
           "color": "green",
           "msg": {
             "type": "Response",
             "id": 60069,
             "value": "Foobar"
           }
         },
         "people": []
       }
    "#};
    let r = Format::Json.load_from_str::<Config>(s);
    assert!(r.is_err());
    assert_eq!(
        r.unwrap_err().to_string(),
        "primitives.integer: invalid type: floating point `3.14`, expected u32 at line 3 column 19"
    );
}
