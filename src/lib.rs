use cfg_if::cfg_if;

#[derive(
    Clone, Copy, Debug, strum::Display, strum::EnumString, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Format {
    Json,
    Toml,
    Yaml,
}

impl Format {
    pub fn is_enabled(&self) -> bool {
        match self {
            Format::Json => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        true
                    } else {
                        false
                    }
                }
            }
            Format::Toml => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        true
                    } else {
                        false
                    }
                }
            }
            Format::Yaml => {
                cfg_if! {
                    if #[cfg(feature = "yaml")] {
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Json => &["json"],
            Format::Toml => &["toml"],
            Format::Yaml => &["yaml", "yml"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod json {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Json;
            assert_eq!(f.to_string(), "json");
            assert_eq!(f.extensions(), ["json"]);
            assert_eq!("json".parse::<Format>().unwrap(), f);
            assert_eq!("JSON".parse::<Format>().unwrap(), f);
            assert_eq!("Json".parse::<Format>().unwrap(), f);
        }

        #[cfg(feature = "json")]
        #[test]
        fn enabled() {
            assert!(Format::Json.is_enabled());
        }

        #[cfg(not(feature = "json"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Json.is_enabled());
        }
    }

    mod toml {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Toml;
            assert_eq!(f.to_string(), "toml");
            assert_eq!(f.extensions(), ["toml"]);
            assert_eq!("toml".parse::<Format>().unwrap(), f);
            assert_eq!("TOML".parse::<Format>().unwrap(), f);
            assert_eq!("Toml".parse::<Format>().unwrap(), f);
        }

        #[cfg(feature = "toml")]
        #[test]
        fn enabled() {
            assert!(Format::Toml.is_enabled());
        }

        #[cfg(not(feature = "toml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Toml.is_enabled());
        }
    }

    mod yaml {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Yaml;
            assert_eq!(f.to_string(), "yaml");
            assert_eq!(f.extensions(), ["yaml", "yml"]);
            assert_eq!("yaml".parse::<Format>().unwrap(), f);
            assert_eq!("YAML".parse::<Format>().unwrap(), f);
            assert_eq!("Yaml".parse::<Format>().unwrap(), f);
        }

        #[cfg(feature = "yaml")]
        #[test]
        fn enabled() {
            assert!(Format::Yaml.is_enabled());
        }

        #[cfg(not(feature = "yaml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Yaml.is_enabled());
        }
    }
}
