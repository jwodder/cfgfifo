mod formats;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct Config {
    primitives: Primitives,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    missing: Option<String>,
    enums: Enums,
    people: Vec<Person>,
}

impl Config {
    fn get() -> Config {
        Config {
            primitives: Primitives::get(),
            missing: None,
            enums: Enums::get(),
            people: Person::get(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct Primitives {
    integer: u32,
    float: f64,
    boolean: bool,
    text: String,
    none: Option<u32>,
    some: Option<u32>,
    list: Vec<u32>,
    dict: BTreeMap<String, String>,
}

impl Primitives {
    fn get() -> Primitives {
        Primitives {
            integer: 42,
            float: 1.618,
            boolean: true,
            text: String::from("This is test text.\nThis is a new line.\n\tThis is an indented line.\nThis is a snowman with a goat: \u{2603}\u{1F410}."),
            none: None,
            some: Some(17),
            list: vec![1, 2, 6, 15, 36],
            dict: BTreeMap::from([
                (String::from("hello"), String::from("goodbye")),
                (String::from("strange"), String::from("charmed")),
                (String::from("up"), String::from("down")),
            ]),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Enums {
    color: Color,
    msg: Message,
}

impl Enums {
    fn get() -> Enums {
        Enums {
            color: Color::Green,
            msg: Message::Response {
                id: 60069,
                value: String::from("Foobar"),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
enum Message {
    Request {
        id: u32,
        resource: String,
        operation: String,
    },
    Response {
        id: u32,
        value: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Person {
    id: u32,
    given_name: String,
    family_name: String,
}

impl Person {
    fn get() -> Vec<Person> {
        vec![
            Person {
                id: 1,
                given_name: String::from("Alice"),
                family_name: String::from("Alison"),
            },
            Person {
                id: 2,
                given_name: String::from("Bob"),
                family_name: String::from("Bobson"),
            },
            Person {
                id: 3,
                given_name: String::from("Charlie"),
                family_name: String::from("McCharles"),
            },
        ]
    }
}
