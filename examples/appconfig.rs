// Run with `cargo run --example appconfig --features examples -- <cfgfile>`
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
