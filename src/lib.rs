use cfg_if::cfg_if;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use strum::{Display, EnumIter, EnumString};
use thiserror::Error;

#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum Format {
    Json,
    Ron,
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
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
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
        // To avoid the need for users to import the trait
        <Format as strum::IntoEnumIterator>::iter()
    }

    pub fn enabled() -> EnabledFormatIter {
        EnabledFormatIter::new()
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Json => &["json"],
            Format::Ron => &["ron"],
            Format::Toml => &["toml"],
            Format::Yaml => &["yaml", "yml"],
        }
    }

    pub fn from_extension(ext: &str) -> Option<Format> {
        let ext = ext.strip_prefix('.').unwrap_or(ext).to_ascii_lowercase();
        let ext = &*ext;
        Format::iter().find(|f| f.extensions().contains(&ext))
    }

    pub fn identify<P: AsRef<Path>>(path: P) -> Result<Format, IdentifyError> {
        let Some(ext) = path.as_ref().extension() else {
            return Err(IdentifyError::NoExtension);
        };
        let Some(ext) = ext.to_str() else {
            return Err(IdentifyError::NotUnicode);
        };
        match Format::from_extension(ext) {
            Some(f) if f.is_enabled() => Ok(f),
            Some(f) => Err(IdentifyError::NotEnabled(f)),
            _ => Err(IdentifyError::Unknown(ext.to_owned())),
        }
    }

    #[allow(unused)]
    pub fn dump_to_string<T: Serialize>(&self, value: &T) -> Result<String, SerializeError> {
        match self {
            Format::Json => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        serde_json::to_string_pretty(value).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
                        ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default()).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Toml => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        toml::to_string_pretty(value).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Yaml => {
                cfg_if! {
                    if #[cfg(feature = "yaml")] {
                        serde_yaml::to_string(value).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
        }
    }

    #[allow(unused)]
    pub fn load_from_str<T: DeserializeOwned>(&self, s: &str) -> Result<T, DeserializeError> {
        match self {
            Format::Json => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        serde_json::from_str(s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
                        ron::from_str(s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Toml => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        toml::from_str(s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Yaml => {
                cfg_if! {
                    if #[cfg(feature = "yaml")] {
                        serde_yaml::from_str(s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
        }
    }

    #[allow(unused)]
    pub fn dump_to_writer<W: io::Write, T: Serialize>(
        &self,
        mut writer: W,
        value: &T,
    ) -> Result<(), SerializeError> {
        match self {
            Format::Json => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        serde_json::to_writer_pretty(&mut writer, value)?;
                        writer.write_all(b"\n")?;
                        Ok(())
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
                        let mut ser = ron::Serializer::new(&mut writer, Some(ron::ser::PrettyConfig::default()))?;
                        value.serialize(&mut ser)?;
                        writer.write_all(b"\n")?;
                        Ok(())
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Toml => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        let s = toml::to_string_pretty(value)?;
                        writer.write_all(s.as_bytes())?;
                        Ok(())
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Yaml => {
                cfg_if! {
                    if #[cfg(feature = "yaml")] {
                        serde_yaml::to_writer(writer, value).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
        }
    }

    #[allow(unused)]
    pub fn load_from_reader<R: io::Read, T: DeserializeOwned>(
        &self,
        mut reader: R,
    ) -> Result<T, DeserializeError> {
        match self {
            Format::Json => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        serde_json::from_reader(reader).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
                        let s = io::read_to_string(reader)?;
                        ron::from_str(&s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Toml => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        let s = io::read_to_string(reader)?;
                        toml::from_str(&s).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Yaml => {
                cfg_if! {
                    if #[cfg(feature = "yaml")] {
                        serde_yaml::from_reader(reader).map_err(Into::into)
                    } else {
                        Err(DeserializeError::NotEnabled(*self))
                    }
                }
            }
        }
    }
}

pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, LoadError> {
    let fmt = Format::identify(&path)?;
    let fp = File::open(path).map_err(LoadError::Open)?;
    fmt.load_from_reader(fp).map_err(Into::into)
}

pub fn dump<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<(), DumpError> {
    let fmt = Format::identify(&path)?;
    let fp = File::create(path).map_err(DumpError::Open)?;
    fmt.dump_to_writer(fp, value).map_err(Into::into)
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifyError {
    #[error("file extension indicates {0}, support for which is not enabled")]
    NotEnabled(Format),
    #[error("unknown file extension: {0:?}")]
    Unknown(String),
    #[error("file extension is not valid Unicode")]
    NotUnicode,
    #[error("file does not have a file extension")]
    NoExtension,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializeError {
    #[error("serialization to {0} is not enabled")]
    NotEnabled(Format),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "ron")]
    #[error(transparent)]
    Ron(#[from] ron::error::Error),
    #[cfg(feature = "toml")]
    #[error(transparent)]
    Toml(#[from] toml::ser::Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeError {
    #[error("deserialization from {0} is not enabled")]
    NotEnabled(Format),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "ron")]
    #[error(transparent)]
    Ron(#[from] ron::error::SpannedError),
    #[cfg(feature = "toml")]
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),
    #[error("failed to open file for reading")]
    Open(#[source] io::Error),
    #[error("failed to deserialize file contents")]
    Deserialize(#[from] DeserializeError),
}

#[derive(Debug, Error)]
pub enum DumpError {
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),
    #[error("failed to open file for writing")]
    Open(#[source] io::Error),
    #[error("failed to serialize structure")]
    Serialize(#[from] SerializeError),
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
        self.0.find(Format::is_enabled)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.0.size_hint();
        (0, upper)
    }
}

impl DoubleEndedIterator for EnabledFormatIter {
    fn next_back(&mut self) -> Option<Format> {
        self.0.rfind(Format::is_enabled)
    }
}

impl std::iter::FusedIterator for EnabledFormatIter {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn in_iter<T, I>(value: T, mut iter: I) -> bool
    where
        T: Eq,
        I: Iterator<Item = T>,
    {
        iter.any(move |v| v == value)
    }

    #[rstest]
    #[case("file.ini", "ini")]
    #[case("file.xml", "xml")]
    #[case("file.cfg", "cfg")]
    #[case("file.jsn", "jsn")]
    #[case("file.tml", "tml")]
    fn identify_unknown(#[case] path: &str, #[case] ext: String) {
        assert_eq!(Format::identify(path), Err(IdentifyError::Unknown(ext)));
    }

    #[cfg(unix)]
    #[test]
    fn identify_not_unicode() {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::OsStr::from_bytes(b"file.js\xF6n");
        assert_eq!(Format::identify(path), Err(IdentifyError::NotUnicode));
    }

    #[cfg(windows)]
    #[test]
    fn identify_not_unicode() {
        use std::os::windows::ffi::OsStringExt;
        let path = std::ffi::OsString::from_wide(&[
            0x66, 0x69, 0x6C, 0x65, 0x2E, 0x6A, 0xDC00, 0x73, 0x6E,
        ]);
        assert_eq!(Format::identify(path), Err(IdentifyError::NotUnicode));
    }

    #[test]
    fn identify_no_ext() {
        assert_eq!(Format::identify("file"), Err(IdentifyError::NoExtension));
    }

    mod json {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Json;
            assert_eq!(f.to_string(), "JSON");
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
            assert!(in_iter(Format::Json, Format::enabled().rev()));
        }

        #[cfg(not(feature = "json"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Json.is_enabled());
            assert!(!in_iter(Format::Json, Format::enabled()));
            assert!(!in_iter(Format::Json, Format::enabled().rev()));
        }

        #[rstest]
        #[case("json")]
        #[case(".json")]
        #[case("JSON")]
        #[case(".JSON")]
        fn from_extension(#[case] ext: &str) {
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Json);
        }

        #[cfg(feature = "json")]
        #[rstest]
        #[case("file.json")]
        #[case("dir/file.JSON")]
        #[case("/dir/file.Json")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Json);
        }

        #[cfg(not(feature = "json"))]
        #[test]
        fn identify_not_enabled() {
            assert_eq!(
                Format::identify("file.json"),
                Err(IdentifyError::NotEnabled(Format::Json))
            );
        }
    }

    mod ron {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Ron;
            assert_eq!(f.to_string(), "RON");
            assert_eq!(f.extensions(), ["ron"]);
            assert_eq!("ron".parse::<Format>().unwrap(), f);
            assert_eq!("RON".parse::<Format>().unwrap(), f);
            assert_eq!("Ron".parse::<Format>().unwrap(), f);
            assert!(in_iter(f, Format::iter()));
        }

        #[cfg(feature = "ron")]
        #[test]
        fn enabled() {
            assert!(Format::Ron.is_enabled());
            assert!(in_iter(Format::Ron, Format::enabled()));
            assert!(in_iter(Format::Ron, Format::enabled().rev()));
        }

        #[cfg(not(feature = "ron"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Ron.is_enabled());
            assert!(!in_iter(Format::Ron, Format::enabled()));
            assert!(!in_iter(Format::Ron, Format::enabled().rev()));
        }

        #[rstest]
        #[case("ron")]
        #[case(".ron")]
        #[case("RON")]
        #[case(".RON")]
        fn from_extension(#[case] ext: &str) {
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Ron);
        }

        #[cfg(feature = "ron")]
        #[rstest]
        #[case("file.ron")]
        #[case("dir/file.RON")]
        #[case("/dir/file.Ron")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Ron);
        }

        #[cfg(not(feature = "ron"))]
        #[test]
        fn identify_not_enabled() {
            assert_eq!(
                Format::identify("file.ron"),
                Err(IdentifyError::NotEnabled(Format::Ron))
            );
        }
    }

    mod toml {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Toml;
            assert_eq!(f.to_string(), "TOML");
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
            assert!(in_iter(Format::Toml, Format::enabled().rev()));
        }

        #[cfg(not(feature = "toml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Toml.is_enabled());
            assert!(!in_iter(Format::Toml, Format::enabled()));
            assert!(!in_iter(Format::Toml, Format::enabled().rev()));
        }

        #[rstest]
        #[case("toml")]
        #[case(".toml")]
        #[case("TOML")]
        #[case(".TOML")]
        fn from_extension(#[case] ext: &str) {
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Toml);
        }

        #[cfg(feature = "toml")]
        #[rstest]
        #[case("file.toml")]
        #[case("dir/file.TOML")]
        #[case("/dir/file.Toml")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Toml);
        }

        #[cfg(not(feature = "toml"))]
        #[test]
        fn identify_not_enabled() {
            assert_eq!(
                Format::identify("file.toml"),
                Err(IdentifyError::NotEnabled(Format::Toml))
            );
        }
    }

    mod yaml {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Yaml;
            assert_eq!(f.to_string(), "YAML");
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
            assert!(in_iter(Format::Yaml, Format::enabled().rev()));
        }

        #[cfg(not(feature = "yaml"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Yaml.is_enabled());
            assert!(!in_iter(Format::Yaml, Format::enabled()));
            assert!(!in_iter(Format::Yaml, Format::enabled().rev()));
        }

        #[rstest]
        #[case("yaml")]
        #[case(".yaml")]
        #[case("YAML")]
        #[case(".YAML")]
        #[case("yml")]
        #[case(".yml")]
        #[case("YML")]
        #[case(".YML")]
        fn from_extension(#[case] ext: &str) {
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Yaml);
        }

        #[cfg(feature = "yaml")]
        #[rstest]
        #[case("file.yaml")]
        #[case("dir/file.YAML")]
        #[case("/dir/file.Yaml")]
        #[case("file.yml")]
        #[case("dir/file.YML")]
        #[case("/dir/file.Yml")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Yaml);
        }

        #[cfg(not(feature = "yaml"))]
        #[test]
        fn identify_not_enabled() {
            assert_eq!(
                Format::identify("file.yaml"),
                Err(IdentifyError::NotEnabled(Format::Yaml))
            );
        }
    }
}
