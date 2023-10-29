#![allow(unused)]
use crate::Config;
use assert_matches::assert_matches;
use cfg_if::cfg_if;
use cfgurate::{DeserializeError, Format, SerializeError};
use indoc::indoc;
use pretty_assertions::assert_eq;

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
    cfg_if! {
        if #[cfg(feature = "toml")] {
            assert_eq!(r.unwrap(), Config::get());
        } else {
            assert_matches!(r, Err(DeserializeError::NotEnabled(Format::Toml)));
        }
    }
}

#[test]
fn dump_to_string() {
    let r = Format::Toml.dump_to_string(&Config::get());
    cfg_if! {
        if #[cfg(feature = "toml")] {
            assert_eq!(r.unwrap(), TOML);
        } else {
            assert_matches!(r, Err(SerializeError::NotEnabled(Format::Toml)));
        }
    }
}
