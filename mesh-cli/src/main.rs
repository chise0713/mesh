mod cli;

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cidr::{Ipv4Cidr, Ipv6Cidr};
use clap::{CommandFactory, FromArgMatches as _};
use cli::{Cli, Commands};
use meshes::{
    conf::Conf,
    mesh::{FromJson as _, Mesh, Meshs, ToJson as _},
};
use x25519_dalek::{PublicKey, StaticSecret};

const IPV4_NETWORK_BROADCAST_OVERHEAD: u32 = 2;
const RESERVED_IPV6_ADDRESS_COUNT: u32 = 1;

fn read_config(path: impl AsRef<str>) -> Result<Meshs> {
    let mut buf = String::with_capacity(4096);
    File::open(path.as_ref())?.read_to_string(&mut buf)?;
    Ok(Meshs::from_json(buf)?)
}

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
                let mut input = String::with_capacity(2);
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
                writeln!(
                    file,
                    "{}",
                    Meshs::new([Mesh::default()], 24, 120).to_json()?
                )?;
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
                    Meshs::new(meshs, ipv4_prefix, ipv6_prefix).to_json()?
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
            let config_map = Conf::default().create_all(read_config(args.config)?)?;
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
        Commands::Append {
            tag,
            in_place,
            count,
        } => {
            let count = count.unwrap_or(1);
            let mut meshs = read_config(&args.config)?;
            let c = meshs.meshs.len() as u32 + count;
            if c > 16_777_214 {
                bail!("Total number of meshes exceed 16,777,214")
            }
            meshs.ipv4_prefix =
                (32 - ((c + IPV4_NETWORK_BROADCAST_OVERHEAD) as f64).log2().ceil() as u8).max(0);
            meshs.ipv6_prefix =
                (128 - ((c + RESERVED_IPV6_ADDRESS_COUNT) as f64).log2().ceil() as u8).max(0);
            let mut max_ipv4 = Ipv4Addr::new(10, 0, 0, 0);
            let mut max_ipv6 = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 0);
            for mesh in meshs.iter() {
                let ipv4 = Ipv4Addr::from_str(&mesh.ipv4).unwrap();
                let ipv6 = Ipv6Addr::from_str(&mesh.ipv6).unwrap();
                if ipv4 > max_ipv4 {
                    max_ipv4 = ipv4;
                }
                if ipv6 > max_ipv6 {
                    max_ipv6 = ipv6;
                }
            }
            let mut rng = rand::thread_rng();
            let mut meshs_vec = meshs.meshs.into_vec();
            for (i, _) in (meshs_vec.len() as u32..c).enumerate() {
                let i = i + 1;
                let ipv4 = Ipv4Addr::from(u32::from(max_ipv4) + i as u32);
                let ipv6 = Ipv6Addr::from(u128::from(max_ipv6) + i as u128);
                let secret = StaticSecret::random_from_rng(&mut rng);
                let public = PublicKey::from(&secret);
                let public = STANDARD.encode(public);
                let secret = STANDARD.encode(secret);
                meshs_vec.push(Mesh::new(
                    if count == 1 {
                        tag.clone()
                    } else {
                        format!("{}-{}", tag, i).into_boxed_str()
                    },
                    public,
                    secret,
                    ipv4.to_string(),
                    ipv6.to_string(),
                    "place.holder.local.arpa:51820",
                ));
            }
            meshs.meshs = meshs_vec.into_boxed_slice();
            let json = meshs.to_json()?;
            if in_place {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&*args.config)?;
                file.set_len(0)?;
                write!(file, "{}", json)?;
            } else {
                println!("{}", json);
            }
        }
    }
    Ok(())
}
