mod cli;

use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cidr::{Ipv4Cidr, Ipv6Cidr};
use clap::{CommandFactory, FromArgMatches as _};
use cli::{Cli, Commands};
use mesh::{
    conf::Conf,
    mesh::{Mesh, Meshs},
};
use x25519_dalek::{PublicKey, StaticSecret};

const IPV4_NETWORK_BROADCAST_OVERHEAD: u32 = 2;
const RESERVED_IPV6_ADDRESS_COUNT: u32 = 1;

fn main() -> Result<()> {
    let mut cmd = Cli::command();
    cmd.build();
    let args = Cli::from_arg_matches(&cmd.clone().get_matches())?;
    match args.command {
        Commands::Init { count } => {
            let path = PathBuf::from(&*args.config);
            if path.exists() {
                eprintln!("Config file already exsits");
                eprint!("continue? [y/N]");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input.len() > 1 {
                    bail!("Invalid input")
                }
                if !input.to_ascii_lowercase().contains('y') {
                    bail!("Aborted")
                }
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&*args.config)?;
            file.set_len(0)?;
            if count.is_none() {
                writeln!(file, "{}", Meshs::new([Mesh::default()], 24, 64).to_json())?;
            } else {
                let count = count.unwrap();
                let ipv4_prefix = (32
                    - ((count + IPV4_NETWORK_BROADCAST_OVERHEAD) as f64)
                        .log2()
                        .ceil() as u8)
                    .max(0);
                let ipv6_prefix = (128
                    - ((count + RESERVED_IPV6_ADDRESS_COUNT) as f64).log2().ceil() as u8)
                    .max(0);
                let mut ipv4 = Ipv4Cidr::new("10.0.0.0".parse()?, ipv4_prefix)?.iter();
                let mut ipv6 = Ipv6Cidr::new("fd00::".parse()?, ipv6_prefix)?.iter();
                ipv4.next().unwrap();
                ipv6.next().unwrap();
                let mut meshs = Vec::with_capacity(count as usize);
                let mut rng = rand::thread_rng();
                for i in 1..=count {
                    let secret = StaticSecret::random_from_rng(&mut rng);
                    let public = PublicKey::from(&secret);
                    let public = STANDARD.encode(public);
                    let secret = STANDARD.encode(secret);
                    meshs.push(Mesh::new(
                        i.to_string(),
                        public,
                        secret,
                        ipv4.next().unwrap().address().to_string(),
                        ipv6.next().unwrap().address().to_string(),
                        "place.holder.local.arpa:51820",
                    ));
                }
                write!(
                    file,
                    "{}",
                    Meshs::new(meshs, ipv4_prefix, ipv6_prefix).to_json()
                )?;
            }
        }
        Commands::Convert { output } => {
            let output = PathBuf::from(&*output);
            if output.is_file() {
                bail!("Output should not be file")
            } else if !output.exists() {
                bail!("Output directory does not exist")
            }
            let mut config = Conf::default();
            let config_map = config.create_all(args.config)?;
            let mut tag_warned = false;
            for (tag, config) in config_map {
                if tag.is_empty() {
                    if !tag_warned {
                        const WARN: &str = "\x1b[0;33mWARNING\x1b[0m";
                        eprintln!(
                            "{}: One or more of the meshes has a empty tag, it will be ignored",
                            WARN
                        );
                        tag_warned = true
                    }
                    continue;
                }
                let path = output.join(format!("{}.conf", tag));
                let mut file = OpenOptions::new().write(true).create(true).open(path)?;
                file.set_len(0)?;
                write!(file, "{}", config)?;
            }
        }
    }
    Ok(())
}
