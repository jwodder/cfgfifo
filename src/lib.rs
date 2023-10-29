#![cfg_attr(docsrs, feature(doc_cfg))]
//! `cfgurate` is a Rust library for serializing & deserializing various common
//! configuration file formats ([JSON][], [JSON5][], [RON][], [TOML][], and
//! [YAML][]), including autodetecting the format of a file based on its file
//! extension.  It's good for application authors who want to support multiple
//! configuration file formats but don't want to write out a bunch of
//! boilerplate.  `cfgurate` has already written that boilerplate for you, so
//! let it (de)serialize your files!
//!
//! [JSON]: https://www.json.org
//! [JSON5]: https://json5.org
//! [RON]: https://github.com/ron-rs/ron
//! [TOML]: https://toml.io
//! [YAML]: https://yaml.org
//!
//! Features
//! ========
//!
//! Support for each configuration file format is controlled by a Cargo
//! feature; the features for all formats are enabled by default.  These
//! features are:
//!
//! - `json` — Support for JSON via the [serde_json] crate
//! - `json5` — Support for JSON5 via the [json5](https://docs.rs/json5) crate
//! - `ron` — Support for RON via the [ron](https://docs.rs/ron) crate
//! - `toml` — Support for TOML via the [toml](https://docs.rs/toml) crate
//! - `yaml` — Support for YAML via the [serde_yaml] crate
//!
//! Format Limitations
//! ==================
//!
//! If you wish to (de)serialize a type in multiple formats using this crate,
//! you must first ensure that all of the formats you're using support the type
//! and its (de)serialization options, as not all formats are equal in this
//! regard.
//!
//! The following format-specific limitations are currently known:
//!
//! - RON does not support internally tagged enums, untagged enums, or the
//!   `serde(flatten)` attribute.
//!
//! - TOML does not support the unit type `()`.
//!
//! - YAML does not support bytes.
//!
//! Example
//! =======
//!
//! ```compile_fail
//! use serde::Deserialize;
//!
//! #[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
//! struct AppConfig {
//!     #[serde(default)]
//!     enable_foo: bool,
//!     #[serde(default)]
//!     bar_type: BarType,
//!     #[serde(default)]
//!     flavor: Option<String>,
//! }
//!
//! #[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
//! enum BarType {
//!     #[default]
//!     Open,
//!     Closed,
//!     Clopen,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let Some(cfgpath) = std::env::args().nth(1) else {
//!         anyhow::bail!("No configuration file specified");
//!     };
//!     // cfgurate identifies the format used by the file `cfgpath` based on its
//!     // file extension and deserializes it appropriately:
//!     let cfg: AppConfig = cfgurate::load(cfgpath)?;
//!     println!("You specified the following configuration:");
//!     println!("{cfg:#?}");
//!     Ok(())
//! }
//! ```

use cfg_if::cfg_if;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use strum::{Display, EnumIter, EnumString};
use thiserror::Error;

#[cfg(feature = "ron")]
use ron::ser::PrettyConfig;

/// An enum of the file formats supported by `cfgurate`.
///
/// All variants are always present, even if support for a given format was
/// disabled at compile time.  To test whether support for a format is enabled,
/// use the [Format::is_enabled] method.
///
/// A Format can be [displayed][std::fmt::Display] as a string containing its
/// name in all-uppercase, and a Format can be [parsed][std::str::FromStr] from
/// its name in any case.
#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum Format {
    /// The [JSON](https://www.json.org) format, (de)serialized with the
    /// [serde_json] crate.
    ///
    /// Serialization uses multiline/"pretty" format.
    Json,

    /// The [JSON5](https://json5.org) format, deserialized with the [json5]
    /// crate.
    ///
    /// Serialization uses multiline/"pretty" format, performed via serde_json,
    /// as json5's serialization (which also uses serde_json) is
    /// single-line/"non-pretty."
    Json5,

    /// The [RON](https://github.com/ron-rs/ron) format, (de)serialized with
    /// the [ron] crate.
    ///
    /// Serialization uses multiline/"pretty" format.
    Ron,

    /// The [TOML](https://toml.io) format, (de)serialized with the [toml]
    /// crate.
    ///
    /// Serialization uses "pretty" format, in which arrays are serialized on
    /// multiple lines.
    Toml,

    /// The [YAML](https://yaml.org) format, (de)serialized with the
    /// [serde_yaml] crate.
    Yaml,
}

impl Format {
    /// Returns true iff support for the format was enabled at compile time via
    /// the relevant Cargo feature
    pub fn is_enabled(&self) -> bool {
        match self {
            Format::Json => cfg!(feature = "json"),
            Format::Json5 => cfg!(feature = "json5"),
            Format::Ron => cfg!(feature = "ron"),
            Format::Toml => cfg!(feature = "toml"),
            Format::Yaml => cfg!(feature = "yaml"),
        }
    }

    /// Returns an iterator over all [Format] variants
    pub fn iter() -> FormatIter {
        // To avoid the need for users to import the trait
        <Format as strum::IntoEnumIterator>::iter()
    }

    /// Returns an iterator over all [enabled][Format::is_enabled] [Format]
    /// variants
    pub fn enabled() -> EnabledFormatIter {
        EnabledFormatIter::new()
    }

    /// Returns an array of the recognized file extensions for the file format.
    ///
    /// File extensions are lowercase and do not start with a period.
    ///
    /// # Example
    ///
    /// ```
    /// use cfgurate::Format;
    ///
    /// assert_eq!(Format::Json.extensions(), &["json"]);
    /// assert_eq!(Format::Yaml.extensions(), &["yaml", "yml"]);
    /// ```
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Json => &["json"],
            Format::Json5 => &["json5"],
            Format::Ron => &["ron"],
            Format::Toml => &["toml"],
            Format::Yaml => &["yaml", "yml"],
        }
    }

    /// Converts a file extension to the corresponding [Format]
    ///
    /// File extensions are matched case-insensitively and may optionally start
    /// with a period.  If the given file extension does not correspond to a
    /// known file format, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use cfgurate::Format;
    ///
    /// assert_eq!(Format::from_extension(".json"), Some(Format::Json));
    /// assert_eq!(Format::from_extension("YML"), Some(Format::Yaml));
    /// assert_eq!(Format::from_extension("cfg"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<Format> {
        let ext = ext.strip_prefix('.').unwrap_or(ext).to_ascii_lowercase();
        let ext = &*ext;
        Format::iter().find(|f| f.extensions().contains(&ext))
    }

    /// Determine the [Format] of a file path based on its file extension.
    ///
    /// Only [enabled][Format::is_enabled] formats are supported by this
    /// method.
    #[cfg_attr(all(feature = "json", feature = "ron"), doc = concat!(
        "# Example\n",
        "\n",
        "```\n",
        "use cfgurate::Format;\n",
        "\n",
        "assert_eq!(Format::identify(\"path/to/file.json\").unwrap(), Format::Json);\n",
        "assert_eq!(Format::identify(\"path/to/file.RON\").unwrap(), Format::Ron);\n",
        "assert!(Format::identify(\"path/to/file.cfg\").is_err());\n",
        "assert!(Format::identify(\"path/to/file\").is_err());\n",
        "```\n",
    ))]
    /// # Errors
    ///
    /// Returns an error if the given file path does not have an extension, the
    /// extension is not valid Unicode, the extension is unknown, or the
    /// extension is for a disabled format.
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

    /// Serialize a value to a string in this format
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not [enabled][Format::is_enabled] or
    /// if the underlying serializer returns an error.
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
            Format::Json5 => {
                cfg_if! {
                    if #[cfg(feature = "json5")] {
                        /// json5::to_string() just serializes as JSON, but
                        /// non-prettily.
                        serde_json::to_string_pretty(value).map_err(Into::into)
                    } else {
                        Err(SerializeError::NotEnabled(*self))
                    }
                }
            }
            Format::Ron => {
                cfg_if! {
                    if #[cfg(feature = "ron")] {
                        ron::ser::to_string_pretty(value, ron_config()).map_err(Into::into)
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

    /// Deserialize a string in this format
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not [enabled][Format::is_enabled] or
    /// if the underlying deserializer returns an error.
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
            Format::Json5 => {
                cfg_if! {
                    if #[cfg(feature = "json5")] {
                        json5::from_str(s).map_err(Into::into)
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

    /// Serialize a value to a [writer][std::io::Write] in this format.
    ///
    /// If the format's serializer does not normally end its output with a
    /// newline, one is appended so that the written text always ends in a
    /// newline.
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not [enabled][Format::is_enabled], if
    /// an I/O error occurs, or if the underlying serializer returns an error.
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
            Format::Json5 => {
                cfg_if! {
                    if #[cfg(feature = "json5")] {
                        // Serialize as JSON, as that's what json5 does, except
                        // the latter doesn't support serializing to a writer.
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
                        let mut ser = ron::Serializer::new(&mut writer, Some(ron_config()))?;
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

    /// Deserialize a value in this format from a [reader][std::io::Read].
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not [enabled][Format::is_enabled], if
    /// an I/O error occurs, or if the underlying deserializer returns an
    /// error.
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
            Format::Json5 => {
                cfg_if! {
                    if #[cfg(feature = "json5")] {
                        let s = io::read_to_string(reader)?;
                        json5::from_str(&s).map_err(Into::into)
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

#[cfg(feature = "ron")]
fn ron_config() -> PrettyConfig {
    // The default PrettyConfig sets new_line to CR LF on Windows.  Let's not
    // do that here.
    PrettyConfig::default().new_line(String::from("\n"))
}

/// Deserialize the contents of the given file, with the format automatically
/// determined based on the file's extension.
///
/// # Errors
///
/// Returns an error if the format cannot be determined from the file
/// extension, if the format is not [enabled][Format::is_enabled], if an I/O
/// error occurs, or if the underlying deserializer returns an error.
pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, LoadError> {
    let fmt = Format::identify(&path)?;
    let fp = File::open(path).map_err(LoadError::Open)?;
    fmt.load_from_reader(fp).map_err(Into::into)
}

/// Serialize a value to the given file, with the format automatically
/// determined based on the file's extension.
///
/// # Errors
///
/// Returns an error if the format cannot be determined from the file
/// extension, if the format is not [enabled][Format::is_enabled], if an I/O
/// error occurs, or if the underlying serializer returns an error.
pub fn dump<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<(), DumpError> {
    let fmt = Format::identify(&path)?;
    let fp = File::create(path).map_err(DumpError::Open)?;
    fmt.dump_to_writer(fp, value).map_err(Into::into)
}

/// Error type returned by [Format::identify]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifyError {
    /// Returned if the file path's extension corresponded to a format that was
    /// not [enabled][Format::is_enabled]
    #[error("file extension indicates {0}, support for which is not enabled")]
    NotEnabled(
        /// The format in question
        Format,
    ),
    /// Returned if the file path's extension did not correspond to a known
    /// file format
    #[error("unknown file extension: {0:?}")]
    Unknown(
        /// The file extension (without leading period)
        String,
    ),
    /// Returned if the file path's extension was not valid Unicode
    #[error("file extension is not valid Unicode")]
    NotUnicode,
    /// Returned if the file path did not have a file extension
    #[error("file does not have a file extension")]
    NoExtension,
}

/// Error type returned by [Format::dump_to_string] and
/// [Format::dump_to_writer]
///
/// The available variants on this enum depend on which formats were enabled at
/// compile time.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializeError {
    /// Returned if the format in question is not [enabled][Format::is_enabled]
    #[error("serialization to {0} is not enabled")]
    NotEnabled(Format),

    /// Returned if an I/O error occurred while writing to a writer.
    ///
    /// Some serializers may catch & report such errors themselves.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Returned if JSON or JSON5 serialization failed
    #[cfg(any(feature = "json", feature = "json5"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "json", feature = "json5"))))]
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Returned if RON serialization failed
    #[cfg(feature = "ron")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
    #[error(transparent)]
    Ron(#[from] ron::error::Error),

    /// Returned if TOML serialization failed
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
    #[error(transparent)]
    Toml(#[from] toml::ser::Error),

    /// Returned if YAML serialization failed
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

/// Error type returned by [Format::load_from_str] and
/// [Format::load_from_reader]
///
/// The available variants on this enum depend on which formats were enabled at
/// compile time.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeError {
    /// Returned if the format in question is not [enabled][Format::is_enabled]
    #[error("deserialization from {0} is not enabled")]
    NotEnabled(Format),

    /// Returned if an I/O error occurred while reading from a reader.
    ///
    /// Some deserializers may catch & report such errors themselves.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Returned if JSON deserialization failed
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Returned if JSON5 deserialization failed
    #[cfg(feature = "json5")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json5")))]
    #[error(transparent)]
    Json5(#[from] json5::Error),

    /// Returned if RON deserialization failed
    #[cfg(feature = "ron")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
    #[error(transparent)]
    Ron(#[from] ron::error::SpannedError),

    /// Returned if TOML deserialization failed
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    /// Returned if YAML deserialization failed
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

/// Error type returned by [load]
#[derive(Debug, Error)]
pub enum LoadError {
    /// Returned if the file format could not be identified from the file
    /// extension or if the format was not [enabled][Format::is_enabled]
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),

    /// Returned if the file could not be opened for reading
    #[error("failed to open file for reading")]
    Open(#[source] io::Error),

    /// Returned if deserialization failed
    #[error("failed to deserialize file contents")]
    Deserialize(#[from] DeserializeError),
}

/// Error type returned by [dump]
#[derive(Debug, Error)]
pub enum DumpError {
    /// Returned if the file format could not be identified from the file
    /// extension or if the format was not [enabled][Format::is_enabled]
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),

    /// Returned if the file could not be opened for writing
    #[error("failed to open file for writing")]
    Open(#[source] io::Error),

    /// Returned if serialization failed
    #[error("failed to serialize structure")]
    Serialize(#[from] SerializeError),
}

/// An iterator over [enabled][Format::is_enabled] [Format] variants
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

    mod json5 {
        use super::*;

        #[test]
        fn basics() {
            let f = Format::Json5;
            assert_eq!(f.to_string(), "JSON5");
            assert_eq!(f.extensions(), ["json5"]);
            assert_eq!("json5".parse::<Format>().unwrap(), f);
            assert_eq!("JSON5".parse::<Format>().unwrap(), f);
            assert_eq!("Json5".parse::<Format>().unwrap(), f);
            assert!(in_iter(f, Format::iter()));
        }

        #[cfg(feature = "json5")]
        #[test]
        fn enabled() {
            assert!(Format::Json5.is_enabled());
            assert!(in_iter(Format::Json5, Format::enabled()));
            assert!(in_iter(Format::Json5, Format::enabled().rev()));
        }

        #[cfg(not(feature = "json5"))]
        #[test]
        fn not_enabled() {
            assert!(!Format::Json5.is_enabled());
            assert!(!in_iter(Format::Json5, Format::enabled()));
            assert!(!in_iter(Format::Json5, Format::enabled().rev()));
        }

        #[rstest]
        #[case("json5")]
        #[case(".json5")]
        #[case("JSON5")]
        #[case(".JSON5")]
        fn from_extension(#[case] ext: &str) {
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Json5);
        }

        #[cfg(feature = "json5")]
        #[rstest]
        #[case("file.json5")]
        #[case("dir/file.JSON5")]
        #[case("/dir/file.Json5")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Json5);
        }

        #[cfg(not(feature = "json5"))]
        #[test]
        fn identify_not_enabled() {
            assert_eq!(
                Format::identify("file.json5"),
                Err(IdentifyError::NotEnabled(Format::Json5))
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
