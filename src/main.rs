use std::{
    char::decode_utf16,
    fs::File,
    io::{BufReader, Read, stdout},
};

use anyhow::{anyhow, Result};
use binread::{BinReaderExt, BinResult};
use clap::Parser;

mod policy_file_entry;
mod key_value_datatype;
mod registry_value;

use csv::Writer;
use policy_file_entry::*;
use simplelog::{TermLogger, Config};


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read
    #[clap()]
    polfile: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

struct U16Reader {
    file: BufReader<File>,
}

impl U16Reader {
    pub fn new(file: File) -> Self {
        Self {
            file: BufReader::new(file),
        }
    }
}

impl Iterator for U16Reader {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: [u8; 2] = [0; 2];
        match self.file.read_exact(&mut buf) {
            Err(_) => None,
            Ok(_) => Some((buf[1] as u16) << 8 | buf[0] as u16),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let _ = TermLogger::init(
        args.verbose.log_level_filter(), 
        Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto);

    let mut polfile = File::open(args.polfile)?;

    let mut header: [u8; 4] = [0; 4];
    polfile.read_exact(&mut header)?;
    if &header != b"PReg" {
        return Err(anyhow!("invalid magic number"));
    }

    let mut version: [u8; 4] = [0; 4];
    polfile.read_exact(&mut version)?;
    if &version != b"\x01\0\0\0" {
        return Err(anyhow!("invalid version number"));
    }

    let mut wtr = Writer::from_writer(stdout());
    
    loop {
        let entry_result: BinResult<PolicyFileEntry> = polfile.read_le();
        match entry_result {
            Ok(entry) => {
                wtr.serialize(entry)?;
            },
            Err(why) => match why {
                binread::Error::Io(why) if why.kind() == std::io::ErrorKind::OutOfMemory => break,
                _ => {
                    log::error!("{why}");
                    //continue;
                    break;
                }
            }
        }
    }
    Ok(())
}
