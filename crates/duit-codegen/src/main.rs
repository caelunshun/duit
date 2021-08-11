use duit_core::spec::Spec;
use std::{fs, path::PathBuf};

#[derive(Debug, argh::FromArgs)]
/// generate Rust code from Duit widget spec files
pub struct Args {
    #[argh(positional)]
    /// input YAML file
    input: PathBuf,
    #[argh(option, short = 'o')]
    /// output Rust file
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let spec = Spec::deserialize_from_str(&fs::read_to_string(&args.input)?)?;

    Ok(())
}
