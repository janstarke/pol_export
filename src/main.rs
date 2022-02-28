use std::{path::{PathBuf, Path}, fs::File, io::Read};

use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read
    #[clap()]
    polfile: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut polfile = File::open(args.polfile)?;

    let mut header:[u8;4] = [0; 4];
    polfile.read_exact(&mut header)?;
    if &header != b"PReg" {
        return Err(anyhow!("invalid magic number"));
    }

    let mut version:[u8;4] = [0; 4];
    polfile.read_exact(&mut version)?;
    if &version != b"\x01\0\0\0" {
        return Err(anyhow!("invalid version number"));
    }

    Ok(())
}
