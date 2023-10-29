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

use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use strum::{Display, EnumIter, EnumString};
use thiserror::Error;

#[cfg(feature = "ron")]
use ron::ser::PrettyConfig;

/// An enum of file formats supported by this build of `cfgurate`.
///
/// Each variant is only present if the corresponding Cargo feature of
/// `cfgurate` was enabled at compile time.
///
/// A Format can be [displayed][std::fmt::Display] as a string containing its
/// name in all-uppercase, and a Format can be [parsed][std::str::FromStr] from
/// its name in any case.
#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum Format {
    /// The [JSON](https://www.json.org) format, (de)serialized with the
    /// [serde_json] crate.
    ///
    /// Serialization uses multiline/"pretty" format.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    Json,

    /// The [JSON5](https://json5.org) format, deserialized with the [json5]
    /// crate.
    ///
    /// Serialization uses multiline/"pretty" format, performed via serde_json,
    /// as json5's serialization (which also uses serde_json) is
    /// single-line/"non-pretty."
    #[cfg(feature = "json5")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json5")))]
    Json5,

    /// The [RON](https://github.com/ron-rs/ron) format, (de)serialized with
    /// the [ron] crate.
    ///
    /// Serialization uses multiline/"pretty" format.
    #[cfg(feature = "ron")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ron")))]
    Ron,

    /// The [TOML](https://toml.io) format, (de)serialized with the [toml]
    /// crate.
    ///
    /// Serialization uses "pretty" format, in which arrays are serialized on
    /// multiple lines.
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
    Toml,

    /// The [YAML](https://yaml.org) format, (de)serialized with the
    /// [serde_yaml] crate.
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    Yaml,
}

impl Format {
    /// Returns an iterator over all [`Format`] variants
    pub fn iter() -> FormatIter {
        // To avoid the need for users to import the trait
        <Format as strum::IntoEnumIterator>::iter()
    }

    /// Returns an array of the recognized file extensions for the file format.
    ///
    /// File extensions are lowercase and do not start with a period.
    #[cfg_attr(all(feature = "json", feature = "yaml"), doc = concat!(
        "# Example\n",
        "\n",
        "```\n",
        "use cfgurate::Format;\n",
        "\n",
        "assert_eq!(Format::Json.extensions(), &[\"json\"]);\n",
        "assert_eq!(Format::Yaml.extensions(), &[\"yaml\", \"yml\"]);\n",
        "```\n",
    ))]
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            #[cfg(feature = "json")]
            Format::Json => &["json"],
            #[cfg(feature = "json5")]
            Format::Json5 => &["json5"],
            #[cfg(feature = "ron")]
            Format::Ron => &["ron"],
            #[cfg(feature = "toml")]
            Format::Toml => &["toml"],
            #[cfg(feature = "yaml")]
            Format::Yaml => &["yaml", "yml"],
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    /// Test whether a file extension is associated with the format
    ///
    /// The file extension is matched case-insensitively may optionally start
    /// with a period.
    #[cfg_attr(feature = "json", doc = concat!(
        "# Example\n",
        "\n",
        "```\n",
        "use cfgurate::Format;\n",
        "\n",
        "assert!(Format::Json.has_extension(\".json\"));\n",
        "assert!(Format::Json.has_extension(\"JSON\"));\n",
        "assert!(!Format::Json.has_extension(\"cfg\"));\n",
        "```\n",
    ))]
    pub fn has_extension(&self, ext: &str) -> bool {
        let ext = ext.strip_prefix('.').unwrap_or(ext);
        self.extensions()
            .iter()
            .any(|x| x.eq_ignore_ascii_case(ext))
    }

    /// Converts a file extension to the corresponding [`Format`]
    ///
    /// File extensions are matched case-insensitively and may optionally start
    /// with a period.  If the given file extension does not correspond to a
    /// known file format, `None` is returned.
    #[cfg_attr(all(feature = "json", feature = "yaml"), doc = concat!(
        "# Example\n",
        "\n",
        "```\n",
        "use cfgurate::Format;\n",
        "\n",
        "assert_eq!(Format::from_extension(\".json\"), Some(Format::Json));\n",
        "assert_eq!(Format::from_extension(\"YML\"), Some(Format::Yaml));\n",
        "assert_eq!(Format::from_extension(\"cfg\"), None);\n",
        "```\n",
    ))]
    pub fn from_extension(ext: &str) -> Option<Format> {
        Format::iter().find(|f| f.has_extension(ext))
    }

    /// Determine the [`Format`] of a file path based on its file extension.
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
    /// extension is not valid Unicode, or the extension is unknown to this
    /// build.
    pub fn identify<P: AsRef<Path>>(path: P) -> Result<Format, IdentifyError> {
        let ext = get_ext(path.as_ref())?;
        Format::from_extension(ext).ok_or_else(|| IdentifyError::Unknown(ext.to_owned()))
    }

    /// Serialize a value to a string in this format
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying serializer returns an error.
    #[allow(unused)]
    pub fn dump_to_string<T: Serialize>(&self, value: &T) -> Result<String, SerializeError> {
        match self {
            #[cfg(feature = "json")]
            Format::Json => serde_json::to_string_pretty(value).map_err(Into::into),
            #[cfg(feature = "json5")]
            Format::Json5 => {
                /// json5::to_string() just serializes as JSON, but
                /// non-prettily.
                serde_json::to_string_pretty(value).map_err(Into::into)
            }
            #[cfg(feature = "ron")]
            Format::Ron => ron::ser::to_string_pretty(value, ron_config()).map_err(Into::into),
            #[cfg(feature = "toml")]
            Format::Toml => toml::to_string_pretty(value).map_err(Into::into),
            #[cfg(feature = "yaml")]
            Format::Yaml => serde_yaml::to_string(value).map_err(Into::into),
            _ => unreachable!(),
        }
    }

    /// Deserialize a string in this format
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying deserializer returns an error.
    #[allow(unused)]
    pub fn load_from_str<T: DeserializeOwned>(&self, s: &str) -> Result<T, DeserializeError> {
        match self {
            #[cfg(feature = "json")]
            Format::Json => serde_json::from_str(s).map_err(Into::into),
            #[cfg(feature = "json5")]
            Format::Json5 => json5::from_str(s).map_err(Into::into),
            #[cfg(feature = "ron")]
            Format::Ron => ron::from_str(s).map_err(Into::into),
            #[cfg(feature = "toml")]
            Format::Toml => toml::from_str(s).map_err(Into::into),
            #[cfg(feature = "yaml")]
            Format::Yaml => serde_yaml::from_str(s).map_err(Into::into),
            _ => unreachable!(),
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
    /// Returns an error if an I/O error occurs or if the underlying serializer
    /// returns an error.
    #[allow(unused)]
    pub fn dump_to_writer<W: io::Write, T: Serialize>(
        &self,
        mut writer: W,
        value: &T,
    ) -> Result<(), SerializeError> {
        match self {
            #[cfg(feature = "json")]
            Format::Json => {
                serde_json::to_writer_pretty(&mut writer, value)?;
                writer.write_all(b"\n")?;
                Ok(())
            }
            #[cfg(feature = "json5")]
            Format::Json5 => {
                // Serialize as JSON, as that's what json5 does, except the
                // latter doesn't support serializing to a writer.
                serde_json::to_writer_pretty(&mut writer, value)?;
                writer.write_all(b"\n")?;
                Ok(())
            }
            #[cfg(feature = "ron")]
            Format::Ron => {
                let mut ser = ron::Serializer::new(&mut writer, Some(ron_config()))?;
                value.serialize(&mut ser)?;
                writer.write_all(b"\n")?;
                Ok(())
            }
            #[cfg(feature = "toml")]
            Format::Toml => {
                let s = toml::to_string_pretty(value)?;
                writer.write_all(s.as_bytes())?;
                Ok(())
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => serde_yaml::to_writer(writer, value).map_err(Into::into),
            _ => unreachable!(),
        }
    }

    /// Deserialize a value in this format from a [reader][std::io::Read].
    ///
    /// # Errors
    ///
    /// Returns an error if an I/O error occurs or if the underlying
    /// deserializer returns an error.
    #[allow(unused)]
    pub fn load_from_reader<R: io::Read, T: DeserializeOwned>(
        &self,
        mut reader: R,
    ) -> Result<T, DeserializeError> {
        match self {
            #[cfg(feature = "json")]
            Format::Json => serde_json::from_reader(reader).map_err(Into::into),
            #[cfg(feature = "json5")]
            Format::Json5 => {
                let s = io::read_to_string(reader)?;
                json5::from_str(&s).map_err(Into::into)
            }
            #[cfg(feature = "ron")]
            Format::Ron => {
                let s = io::read_to_string(reader)?;
                ron::from_str(&s).map_err(Into::into)
            }
            #[cfg(feature = "toml")]
            Format::Toml => {
                let s = io::read_to_string(reader)?;
                toml::from_str(&s).map_err(Into::into)
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => serde_yaml::from_reader(reader).map_err(Into::into),
            _ => unreachable!(),
        }
    }
}

/// Deserialize the contents of the given file, with the format automatically
/// determined based on the file's extension.
///
/// # Errors
///
/// Returns an error if the format cannot be determined from the file
/// extension, if an I/O error occurs, or if the underlying deserializer
/// returns an error.
pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, LoadError> {
    Cfgurate::default().load(path)
}

/// Serialize a value to the given file, with the format automatically
/// determined based on the file's extension.
///
/// # Errors
///
/// Returns an error if the format cannot be determined from the file
/// extension, if an I/O error occurs, or if the underlying serializer returns
/// an error.
pub fn dump<P: AsRef<Path>, T: Serialize>(path: P, value: &T) -> Result<(), DumpError> {
    Cfgurate::default().dump(path, value)
}

/// A configurable loader & dumper of serialized data in files.
///
/// By default, a `Cfgurate` instance's [`identify()`][Cfgurate::identify],
/// [`load()`][Cfgurate::load], and [`dump()`][Cfgurate::dump] methods act the
/// same as [`Format::identify()`], [`load()`], and [`dump()`], but the
/// instance can be customized to only support a subset of enabled [`Format`]s
/// and/or to use a given fallback [`Format`] if identifying a file's format
/// fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cfgurate {
    formats: Vec<Format>,
    fallback: Option<Format>,
}

impl Cfgurate {
    /// Create a new Cfgurate instance
    pub fn new() -> Cfgurate {
        Cfgurate {
            formats: Format::iter().collect(),
            fallback: None,
        }
    }

    /// Set the [`Format`]s to support.
    ///
    /// By default, all enabled formats are selected.
    ///
    /// This is useful if you want to always restrict loading & dumping to a
    /// certain set of formats even if more formats become enabled via [feature
    /// unification].
    ///
    /// [feature unification]: https://doc.rust-lang.org/cargo/reference/features.html#feature-unification
    pub fn formats<I: IntoIterator<Item = Format>>(mut self, iter: I) -> Self {
        self.formats = iter.into_iter().collect();
        self
    }

    /// Set a fallback [`Format`] to use if file format identification fails
    pub fn fallback(mut self, fallback: Option<Format>) -> Self {
        self.fallback = fallback;
        self
    }

    /// Determine the [`Format`] of a file path based on its file extension.
    #[cfg_attr(all(feature = "json", feature = "yaml"), doc = concat!(
        "# Example\n",
        "\n",
        "```\n",
        "use cfgurate::{Cfgurate, Format};\n",
        "\n",
        "let cfgurate = Cfgurate::new()\n",
        "    .formats([Format::Json, Format::Yaml])\n",
        "    .fallback(Some(Format::Json));\n",
        "\n",
        "assert_eq!(cfgurate.identify(\"path/to/file.json\").unwrap(), Format::Json);\n",
        "assert_eq!(cfgurate.identify(\"path/to/file.YML\").unwrap(), Format::Yaml);\n",
        "assert_eq!(cfgurate.identify(\"path/to/file.ron\").unwrap(), Format::Json);\n",
        "assert_eq!(cfgurate.identify(\"path/to/file.cfg\").unwrap(), Format::Json);\n",
        "assert_eq!(cfgurate.identify(\"path/to/file\").unwrap(), Format::Json);\n",
        "```\n",
    ))]
    /// # Errors
    ///
    /// Returns an error if the given file path does not have an extension, the
    /// extension is not valid Unicode, or the extension does not belong to a
    /// supported [`Format`].
    ///
    /// All error conditions are suppressed if a [fallback][Cfgurate::fallback]
    /// was set.
    pub fn identify<P: AsRef<Path>>(&self, path: P) -> Result<Format, IdentifyError> {
        let ext = match (get_ext(path.as_ref()), self.fallback) {
            (Ok(ext), _) => ext,
            (Err(_), Some(f)) => return Ok(f),
            (Err(e), _) => return Err(e),
        };
        self.formats
            .iter()
            .find(|f| f.has_extension(ext))
            .copied()
            .or(self.fallback)
            .ok_or_else(|| IdentifyError::Unknown(ext.to_owned()))
    }

    /// Deserialize the contents of the given file, with the format
    /// automatically determined based on the file's extension.
    ///
    /// # Errors
    ///
    /// Returns an error if the format cannot be determined from the file
    /// extension and no fallback format was set, if an I/O error occurs, or if
    /// the underlying deserializer returns an error.
    pub fn load<T: DeserializeOwned, P: AsRef<Path>>(&self, path: P) -> Result<T, LoadError> {
        let fmt = self.identify(&path)?;
        let fp = File::open(path).map_err(LoadError::Open)?;
        fmt.load_from_reader(fp).map_err(Into::into)
    }

    /// Serialize a value to the given file, with the format automatically
    /// determined based on the file's extension.
    ///
    /// # Errors
    ///
    /// Returns an error if the format cannot be determined from the file
    /// extension and no fallback format was set, if an I/O error occurs, or if
    /// the underlying serializer returns an error.
    pub fn dump<P: AsRef<Path>, T: Serialize>(&self, path: P, value: &T) -> Result<(), DumpError> {
        let fmt = self.identify(&path)?;
        let fp = File::create(path).map_err(DumpError::Open)?;
        fmt.dump_to_writer(fp, value).map_err(Into::into)
    }
}

impl Default for Cfgurate {
    /// Same as [`Cfgurate::new()`]
    fn default() -> Cfgurate {
        Cfgurate::new()
    }
}

/// Error type returned by [`Format::identify()`] and [`Cfgurate::identify()`]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifyError {
    /// Returned if the file path's extension did not correspond to a known &
    /// enabled file format
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

/// Error type returned by [`Format::dump_to_string()`] and
/// [`Format::dump_to_writer()`]
///
/// The available variants on this enum depend on which formats were enabled at
/// compile time.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializeError {
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

/// Error type returned by [`Format::load_from_str()`] and
/// [`Format::load_from_reader()`]
///
/// The available variants on this enum depend on which formats were enabled at
/// compile time.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeError {
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

/// Error type returned by [`load()`] and [`Cfgurate::load()`]
#[derive(Debug, Error)]
pub enum LoadError {
    /// Returned if the file format could not be identified from the file
    /// extension
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),

    /// Returned if the file could not be opened for reading
    #[error("failed to open file for reading")]
    Open(#[source] io::Error),

    /// Returned if deserialization failed
    #[error("failed to deserialize file contents")]
    Deserialize(#[from] DeserializeError),
}

/// Error type returned by [`dump()`] and [`Cfgurate::dump()`]
#[derive(Debug, Error)]
pub enum DumpError {
    /// Returned if the file format could not be identified from the file
    /// extension
    #[error("failed to identify file format")]
    Identify(#[from] IdentifyError),

    /// Returned if the file could not be opened for writing
    #[error("failed to open file for writing")]
    Open(#[source] io::Error),

    /// Returned if serialization failed
    #[error("failed to serialize structure")]
    Serialize(#[from] SerializeError),
}

#[cfg(feature = "ron")]
fn ron_config() -> PrettyConfig {
    // The default PrettyConfig sets new_line to CR LF on Windows.  Let's not
    // do that here.
    PrettyConfig::default().new_line(String::from("\n"))
}

fn get_ext(path: &Path) -> Result<&str, IdentifyError> {
    path.extension()
        .ok_or(IdentifyError::NoExtension)?
        .to_str()
        .ok_or(IdentifyError::NotUnicode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("file.ini", "ini")]
    #[case("file.xml", "xml")]
    #[case("file.cfg", "cfg")]
    #[case("file.jsn", "jsn")]
    #[case("file.tml", "tml")]
    fn identify_unknown(#[case] path: &str, #[case] ext: String) {
        assert_eq!(
            Format::identify(path),
            Err(IdentifyError::Unknown(ext.clone()))
        );
        assert_eq!(
            Cfgurate::default().identify(path),
            Err(IdentifyError::Unknown(ext))
        );
    }

    #[cfg(unix)]
    #[test]
    fn identify_not_unicode() {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::OsStr::from_bytes(b"file.js\xF6n");
        assert_eq!(Format::identify(path), Err(IdentifyError::NotUnicode));
        assert_eq!(
            Cfgurate::default().identify(path),
            Err(IdentifyError::NotUnicode)
        );
    }

    #[cfg(windows)]
    #[test]
    fn identify_not_unicode() {
        use std::os::windows::ffi::OsStringExt;
        let path = std::ffi::OsString::from_wide(&[
            0x66, 0x69, 0x6C, 0x65, 0x2E, 0x6A, 0xDC00, 0x73, 0x6E,
        ]);
        assert_eq!(Format::identify(&path), Err(IdentifyError::NotUnicode));
        assert_eq!(
            Cfgurate::default().identify(path),
            Err(IdentifyError::NotUnicode)
        );
    }

    #[test]
    fn identify_no_ext() {
        assert_eq!(Format::identify("file"), Err(IdentifyError::NoExtension));
        assert_eq!(
            Cfgurate::default().identify("file"),
            Err(IdentifyError::NoExtension)
        );
    }

    #[cfg(feature = "json")]
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
            assert!(Format::iter().any(|f2| f == f2));
        }

        #[rstest]
        #[case("json")]
        #[case(".json")]
        #[case("JSON")]
        #[case(".JSON")]
        fn from_extension(#[case] ext: &str) {
            assert!(Format::Json.has_extension(ext));
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Json);
        }

        #[rstest]
        #[case("file.json")]
        #[case("dir/file.JSON")]
        #[case("/dir/file.Json")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Json);
        }
    }

    #[cfg(not(feature = "json"))]
    mod not_json {
        use super::*;

        #[test]
        fn not_variant() {
            assert!(!Format::iter().any(|f| f.to_string() == "JSON"));
        }

        #[test]
        fn identify() {
            assert_eq!(
                Format::identify("file.json"),
                Err(IdentifyError::Unknown(String::from("json")))
            );
        }
    }

    #[cfg(feature = "json5")]
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
            assert!(Format::iter().any(|f2| f == f2));
        }

        #[rstest]
        #[case("json5")]
        #[case(".json5")]
        #[case("JSON5")]
        #[case(".JSON5")]
        fn from_extension(#[case] ext: &str) {
            assert!(Format::Json5.has_extension(ext));
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Json5);
        }

        #[rstest]
        #[case("file.json5")]
        #[case("dir/file.JSON5")]
        #[case("/dir/file.Json5")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Json5);
        }
    }

    #[cfg(not(feature = "json5"))]
    mod not_json5 {
        use super::*;

        #[test]
        fn not_variant() {
            assert!(!Format::iter().any(|f| f.to_string() == "JSON5"));
        }

        #[test]
        fn identify() {
            assert_eq!(
                Format::identify("file.json5"),
                Err(IdentifyError::Unknown(String::from("json5")))
            );
        }
    }

    #[cfg(feature = "ron")]
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
            assert!(Format::iter().any(|f2| f == f2));
        }

        #[rstest]
        #[case("ron")]
        #[case(".ron")]
        #[case("RON")]
        #[case(".RON")]
        fn from_extension(#[case] ext: &str) {
            assert!(Format::Ron.has_extension(ext));
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Ron);
        }

        #[rstest]
        #[case("file.ron")]
        #[case("dir/file.RON")]
        #[case("/dir/file.Ron")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Ron);
        }
    }

    #[cfg(not(feature = "ron"))]
    mod not_ron {
        use super::*;

        #[test]
        fn not_variant() {
            assert!(!Format::iter().any(|f| f.to_string() == "RON"));
        }

        #[test]
        fn identify() {
            assert_eq!(
                Format::identify("file.ron"),
                Err(IdentifyError::Unknown(String::from("ron")))
            );
        }
    }

    #[cfg(feature = "toml")]
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
            assert!(Format::iter().any(|f2| f == f2));
        }

        #[rstest]
        #[case("toml")]
        #[case(".toml")]
        #[case("TOML")]
        #[case(".TOML")]
        fn from_extension(#[case] ext: &str) {
            assert!(Format::Toml.has_extension(ext));
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Toml);
        }

        #[rstest]
        #[case("file.toml")]
        #[case("dir/file.TOML")]
        #[case("/dir/file.Toml")]
        fn identify(#[case] path: &str) {
            assert_eq!(Format::identify(path).unwrap(), Format::Toml);
        }
    }

    #[cfg(not(feature = "toml"))]
    mod not_toml {
        use super::*;

        #[test]
        fn not_variant() {
            assert!(!Format::iter().any(|f| f.to_string() == "TOML"));
        }

        #[test]
        fn identify() {
            assert_eq!(
                Format::identify("file.toml"),
                Err(IdentifyError::Unknown(String::from("toml")))
            );
        }
    }

    #[cfg(feature = "yaml")]
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
            assert!(Format::iter().any(|f2| f == f2));
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
            assert!(Format::Yaml.has_extension(ext));
            assert_eq!(Format::from_extension(ext).unwrap(), Format::Yaml);
        }

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
    }

    #[cfg(not(feature = "yaml"))]
    mod not_yaml {
        use super::*;

        #[test]
        fn not_variant() {
            assert!(!Format::iter().any(|f| f.to_string() == "YAML"));
        }

        #[test]
        fn identify() {
            assert_eq!(
                Format::identify("file.yaml"),
                Err(IdentifyError::Unknown(String::from("yaml")))
            );
        }
    }

    mod cfgurate {
        #[allow(unused_imports)]
        use super::*;

        #[cfg(all(
            feature = "json",
            feature = "json5",
            feature = "ron",
            feature = "toml",
            feature = "yaml"
        ))]
        #[test]
        fn default() {
            let cfg = Cfgurate::default();
            assert_eq!(cfg.identify("file.json").unwrap(), Format::Json);
            assert_eq!(cfg.identify("file.json5").unwrap(), Format::Json5);
            assert_eq!(cfg.identify("file.Ron").unwrap(), Format::Ron);
            assert_eq!(cfg.identify("file.toml").unwrap(), Format::Toml);
            assert_eq!(cfg.identify("file.YML").unwrap(), Format::Yaml);
            assert!(cfg.identify("file.cfg").is_err());
            assert!(cfg.identify("file").is_err());
        }

        #[cfg(all(
            feature = "json",
            feature = "json5",
            feature = "ron",
            feature = "toml",
            feature = "yaml"
        ))]
        #[test]
        fn fallback() {
            let cfg = Cfgurate::new().fallback(Some(Format::Json));
            assert_eq!(cfg.identify("file.json").unwrap(), Format::Json);
            assert_eq!(cfg.identify("file.json5").unwrap(), Format::Json5);
            assert_eq!(cfg.identify("file.Ron").unwrap(), Format::Ron);
            assert_eq!(cfg.identify("file.toml").unwrap(), Format::Toml);
            assert_eq!(cfg.identify("file.YML").unwrap(), Format::Yaml);
            assert_eq!(cfg.identify("file.cfg").unwrap(), Format::Json);
            assert_eq!(cfg.identify("file").unwrap(), Format::Json);
        }

        #[cfg(all(feature = "json", feature = "toml"))]
        #[test]
        fn formats() {
            let cfg = Cfgurate::new().formats([Format::Json, Format::Toml]);
            assert_eq!(cfg.identify("file.json").unwrap(), Format::Json);
            assert!(cfg.identify("file.json5").is_err());
            assert!(cfg.identify("file.Ron").is_err());
            assert_eq!(cfg.identify("file.toml").unwrap(), Format::Toml);
            assert!(cfg.identify("file.YML").is_err());
            assert!(cfg.identify("file.cfg").is_err());
            assert!(cfg.identify("file").is_err());
        }

        #[cfg(all(feature = "json", feature = "toml", feature = "yaml"))]
        #[test]
        fn formats_fallback() {
            let cfg = Cfgurate::new()
                .formats([Format::Json, Format::Toml])
                .fallback(Some(Format::Yaml));
            assert_eq!(cfg.identify("file.json").unwrap(), Format::Json);
            assert_eq!(cfg.identify("file.json5").unwrap(), Format::Yaml);
            assert_eq!(cfg.identify("file.Ron").unwrap(), Format::Yaml);
            assert_eq!(cfg.identify("file.toml").unwrap(), Format::Toml);
            assert_eq!(cfg.identify("file.YML").unwrap(), Format::Yaml);
            assert_eq!(cfg.identify("file.cfg").unwrap(), Format::Yaml);
            assert_eq!(cfg.identify("file").unwrap(), Format::Yaml);
        }
    }
}
