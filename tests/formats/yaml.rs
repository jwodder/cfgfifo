#![allow(unused)]
use crate::Config;
use assert_matches::assert_matches;
use cfg_if::cfg_if;
use cfgurate::{DeserializeError, Format, SerializeError};
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::{read_to_string, Read, Seek, Write};
use tempfile::tempfile;

static YAML: &str = indoc! {r#"
primitives:
  integer: 42
  float: 1.618
  boolean: true
  text: "This is test text.\nThis is a new line.\n\tThis is an indented line.\nThis is a snowman with a goat: ☃🐐."
  none: null
  some: 17
  list:
  - 1
  - 2
  - 6
  - 15
  - 36
  dict:
    hello: goodbye
    strange: charmed
    up: down
enums:
  color: green
  msg:
    type: Response
    id: 60069
    value: Foobar
people:
- id: 1
  given_name: Alice
  family_name: Alison
- id: 2
  given_name: Bob
  family_name: Bobson
- id: 3
  given_name: Charlie
  family_name: McCharles
"#};

#[test]
fn load_from_str() {
    let r = Format::Yaml.load_from_str::<Config>(YAML);
    cfg_if! {
        if #[cfg(feature = "yaml")] {
            assert_eq!(r.unwrap(), Config::get());
        } else {
            assert_matches!(r, Err(DeserializeError::NotEnabled(Format::Yaml)));
        }
    }
}

#[test]
fn dump_to_string() {
    let r = Format::Yaml.dump_to_string(&Config::get());
    cfg_if! {
        if #[cfg(feature = "yaml")] {
            assert_eq!(r.unwrap(), YAML);
        } else {
            assert_matches!(r, Err(SerializeError::NotEnabled(Format::Yaml)));
        }
    }
}

#[test]
fn load_from_reader() {
    let mut file = tempfile().unwrap();
    file.write_all(YAML.as_bytes()).unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = Format::Yaml.load_from_reader::<_, Config>(file);
    cfg_if! {
        if #[cfg(feature = "yaml")] {
            assert_eq!(r.unwrap(), Config::get());
        } else {
            assert_matches!(r, Err(DeserializeError::NotEnabled(Format::Yaml)));
        }
    }
}

#[test]
fn dump_to_writer() {
    let mut file = tempfile().unwrap();
    let r = Format::Yaml.dump_to_writer(&file, &Config::get());
    cfg_if! {
        if #[cfg(feature = "yaml")] {
            assert!(r.is_ok());
            file.flush().unwrap();
            file.rewind().unwrap();
            let s = read_to_string(file).unwrap();
            assert_eq!(s, YAML);
            assert!(s.ends_with("McCharles\n"));
        } else {
            assert_matches!(r, Err(SerializeError::NotEnabled(Format::Yaml)));
        }
    }
}
