#![cfg(feature = "yaml")]
use crate::Config;
use cfgfifo::*;
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::{read_to_string, Seek, Write};
use tempfile::{tempfile, Builder};

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
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_string() {
    let r = Format::Yaml.dump_to_string(&Config::get());
    assert_eq!(r.unwrap(), YAML);
}

#[test]
fn load_from_reader() {
    let mut file = tempfile().unwrap();
    file.write_all(YAML.as_bytes()).unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = Format::Yaml.load_from_reader::<_, Config>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_writer() {
    let mut file = tempfile().unwrap();
    let r = Format::Yaml.dump_to_writer(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, YAML);
    assert!(s.ends_with("McCharles\n"));
}

#[test]
fn load_from_file() {
    let mut file = Builder::new().suffix(".yaml").tempfile().unwrap();
    file.write_all(YAML.as_bytes()).unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = load::<Config, _>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_file() {
    let mut file = Builder::new().suffix(".yaml").tempfile().unwrap();
    let r = dump(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, YAML);
    assert!(s.ends_with("McCharles\n"));
}
