#![allow(unused)]
use crate::Config;
use assert_matches::assert_matches;
use cfg_if::cfg_if;
use cfgurate::{DeserializeError, Format, SerializeError};
use indoc::indoc;
use pretty_assertions::assert_eq;

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
    cfg_if! {
        if #[cfg(feature = "json")] {
            assert_eq!(r.unwrap(), Config::get());
        } else {
            assert_matches!(r, Err(DeserializeError::NotEnabled(Format::Json)));
        }
    }
}

#[test]
fn dump_to_string() {
    let r = Format::Json.dump_to_string(&Config::get());
    cfg_if! {
        if #[cfg(feature = "json")] {
            assert_eq!(r.unwrap(), JSON);
        } else {
            assert_matches!(r, Err(SerializeError::NotEnabled(Format::Json)));
        }
    }
}
