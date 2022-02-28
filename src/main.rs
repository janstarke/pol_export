use std::{fs::File, io::{Read, BufReader}, char::{decode_utf16, DecodeUtf16Error}};

use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read
    #[clap()]
    polfile: String,
}

struct U16Reader {
    file: BufReader<File>
}

impl U16Reader {
    pub fn new(file: File) -> Self {
        Self {
            file: BufReader::new(file)
        }
    }
}

impl Iterator for U16Reader {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf:[u8;2] = [0;2];
        match self.file.read_exact(&mut buf) {
            Err(_) => None,
            Ok(_) => Some((buf[1] as u16) << 8 | buf[0] as u16)
        }
    }
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

    let reader = U16Reader::new(polfile);
    let content: String = decode_utf16(reader)
        .map(|r| r.or(Ok('?')))
        .map(|r: Result<char, DecodeUtf16Error>| r.unwrap())
        .collect();
    let content = content.replace("][", "]\n[");

    println!("{}", content);
    Ok(())
}