#![cfg(feature = "toml")]
use crate::Config;
use cfgfifo::*;
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::{read_to_string, Seek, Write};
use tempfile::{tempfile, Builder};

static TOML: &str = indoc! {r#"
[primitives]
integer = 42
float = 1.618
boolean = true
text = """
This is test text.
This is a new line.
\tThis is an indented line.
This is a snowman with a goat: ☃🐐."""
some = 17
list = [
    1,
    2,
    6,
    15,
    36,
]

[primitives.dict]
hello = "goodbye"
strange = "charmed"
up = "down"

[enums]
color = "green"

[enums.msg]
type = "Response"
id = 60069
value = "Foobar"

[[people]]
id = 1
given_name = "Alice"
family_name = "Alison"

[[people]]
id = 2
given_name = "Bob"
family_name = "Bobson"

[[people]]
id = 3
given_name = "Charlie"
family_name = "McCharles"
"#};

#[test]
fn load_from_str() {
    let r = Format::Toml.load_from_str::<Config>(TOML);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_string() {
    let r = Format::Toml.dump_to_string(&Config::get());
    assert_eq!(r.unwrap(), TOML);
}

#[test]
fn load_from_reader() {
    let mut file = tempfile().unwrap();
    file.write_all(TOML.as_bytes()).unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = Format::Toml.load_from_reader::<_, Config>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_writer() {
    let mut file = tempfile().unwrap();
    let r = Format::Toml.dump_to_writer(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, TOML);
    assert!(s.ends_with("\"\n"));
}

#[test]
fn load_from_file() {
    let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
    file.write_all(TOML.as_bytes()).unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = load::<Config, _>(file);
    assert_eq!(r.unwrap(), Config::get());
}

#[test]
fn dump_to_file() {
    let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
    let r = dump(&file, &Config::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, TOML);
    assert!(s.ends_with("\"\n"));
}
