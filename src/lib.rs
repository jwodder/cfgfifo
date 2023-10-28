use cfg_if::cfg_if;

#[derive(
    Clone,
    Copy,
    Debug,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
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

    pub fn iter() -> FormatIter {
        <Format as strum::IntoEnumIterator>::iter()
    }

    pub fn enabled() -> EnabledFormatIter {
        EnabledFormatIter::new()
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Json => &["json"],
            Format::Toml => &["toml"],
            Format::Yaml => &["yaml", "yml"],
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnabledFormatIter(FormatIter);

impl EnabledFormatIter {
    pub fn new() -> EnabledFormatIter {
        EnabledFormatIter(Format::iter())
    }
}

impl Default for EnabledFormatIter {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for EnabledFormatIter {
    type Item = Format;

    fn next(&mut self) -> Option<Format> {
        self.0.find(|f| f.is_enabled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_iter<T, I>(value: T, mut iter: I) -> bool
    where
        T: Eq,
        I: Iterator<Item = T>,
    {
        iter.any(move |v| v == value)
    }

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
            assert!(in_iter(f, Format::iter()));
        }

        #[cfg(feature = "json")]
        #[test]
        fn enabled() {
            assert!(Format::Json.is_enabled());
            assert!(in_iter(Format::Json, Format::enabled()));
        }

        #[cfg(not(feature = "json"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Json.is_enabled());
            assert!(!in_iter(Format::Json, Format::enabled()));
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
            assert!(in_iter(f, Format::iter()));
        }

        #[cfg(feature = "toml")]
        #[test]
        fn enabled() {
            assert!(Format::Toml.is_enabled());
            assert!(in_iter(Format::Toml, Format::enabled()));
        }

        #[cfg(not(feature = "toml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Toml.is_enabled());
            assert!(!in_iter(Format::Toml, Format::enabled()));
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
            assert!(in_iter(f, Format::iter()));
        }

        #[cfg(feature = "yaml")]
        #[test]
        fn enabled() {
            assert!(Format::Yaml.is_enabled());
            assert!(in_iter(Format::Yaml, Format::enabled()));
        }

        #[cfg(not(feature = "yaml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Yaml.is_enabled());
            assert!(!in_iter(Format::Yaml, Format::enabled()));
        }
    }
}
