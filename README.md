[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![CI Status](https://github.com/jwodder/cfgurate/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/cfgurate/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/cfgurate/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/cfgurate)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.67-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/cfgurate.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/cfgurate) | [Issues](https://github.com/jwodder/cfgurate/issues)

`cfgurate` is a Rust library for serializing & deserializing various common
configuration file formats ([JSON][], [JSON5][], [RON][], [TOML][], and
[YAML][]), including autodetecting the format of a file based on its file
extension.  It's good for application authors who want to support multiple
configuration file formats but don't want to write out a bunch of boilerplate.
`cfgurate` has already written that boilerplate for you, so let it
(de)serialize your files!

[JSON]: https://www.json.org
[JSON5]: https://json5.org
[RON]: https://github.com/ron-rs/ron
[TOML]: https://toml.io
[YAML]: https://yaml.org

Example
=======

```rust
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct AppConfig {
    #[serde(default)]
    enable_foo: bool,
    #[serde(default)]
    bar_type: BarType,
    #[serde(default)]
    flavor: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
enum BarType {
    #[default]
    Open,
    Closed,
    Clopen,
}

fn main() -> anyhow::Result<()> {
    let Some(cfgpath) = std::env::args().nth(1) else {
        anyhow::bail!("No configuration file specified");
    };
    // cfgurate identifies the format used by the file `cfgpath` based on its
    // file extension and deserializes it appropriately:
    let cfg: AppConfig = cfgurate::load(cfgpath)?;
    println!("You specified the following configuration:");
    println!("{cfg:#?}");
    Ok(())
}
```
