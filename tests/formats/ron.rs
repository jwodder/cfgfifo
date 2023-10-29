#![cfg(feature = "ron")]
use crate::RonConfig;
use cfgurate::*;
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::{read_to_string, Seek, Write};
use tempfile::{tempfile, Builder};

static RON: &str = indoc! {r#"
(
    primitives: (
        integer: 42,
        float: 1.618,
        boolean: true,
        text: "This is test text.\nThis is a new line.\n\tThis is an indented line.\nThis is a snowman with a goat: ☃🐐.",
        none: None,
        some: Some(17),
        list: [
            1,
            2,
            6,
            15,
            36,
        ],
        dict: {
            "hello": "goodbye",
            "strange": "charmed",
            "up": "down",
        },
    ),
    enums: (
        color: green,
        msg: Response(
            id: 60069,
            value: "Foobar",
        ),
    ),
    people: [
        (
            id: 1,
            given_name: "Alice",
            family_name: "Alison",
        ),
        (
            id: 2,
            given_name: "Bob",
            family_name: "Bobson",
        ),
        (
            id: 3,
            given_name: "Charlie",
            family_name: "McCharles",
        ),
    ],
)"#};

#[test]
fn load_from_str() {
    let r = Format::Ron.load_from_str::<RonConfig>(RON);
    assert_eq!(r.unwrap(), RonConfig::get());
}

#[test]
fn dump_to_string() {
    let r = Format::Ron.dump_to_string(&RonConfig::get());
    assert_eq!(r.unwrap(), RON);
}

#[test]
fn load_from_reader() {
    let mut file = tempfile().unwrap();
    writeln!(file, "{RON}").unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = Format::Ron.load_from_reader::<_, RonConfig>(file);
    assert_eq!(r.unwrap(), RonConfig::get());
}

#[test]
fn dump_to_writer() {
    let mut file = tempfile().unwrap();
    let r = Format::Ron.dump_to_writer(&file, &RonConfig::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, format!("{RON}\n"));
    assert!(s.ends_with(")\n"));
}

#[test]
fn load_from_file() {
    let mut file = Builder::new().suffix(".ron").tempfile().unwrap();
    writeln!(file, "{RON}").unwrap();
    file.flush().unwrap();
    file.rewind().unwrap();
    let r = load::<RonConfig, _>(file);
    assert_eq!(r.unwrap(), RonConfig::get());
}

#[test]
fn dump_to_file() {
    let mut file = Builder::new().suffix(".ron").tempfile().unwrap();
    let r = dump(&file, &RonConfig::get());
    assert!(r.is_ok());
    file.flush().unwrap();
    file.rewind().unwrap();
    let s = read_to_string(file).unwrap();
    assert_eq!(s, format!("{RON}\n"));
    assert!(s.ends_with(")\n"));
}
