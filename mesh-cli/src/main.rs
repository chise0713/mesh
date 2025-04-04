mod cli;

use std::{
    collections::HashSet,
    fs,
    hash::Hash,
    io,
    net::{Ipv4Addr, Ipv6Addr},
    ops::{Add, BitAnd, Not, Shl, Sub},
    path::Path,
    str::FromStr,
};

use anyhow::{Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
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

fn read_config(path: impl AsRef<Path>) -> Result<Meshs> {
    let buf = fs::read_to_string(path.as_ref())?;
    Ok(Meshs::from_json(buf)?)
}

trait TruncateToUsize {
    fn truncate_to_usize(self) -> usize;
}

impl TruncateToUsize for u32 {
    fn truncate_to_usize(self) -> usize {
        self as usize
    }
}

impl TruncateToUsize for u128 {
    fn truncate_to_usize(self) -> usize {
        self as usize
    }
}

trait Ip: Copy + Eq + Hash + Ord {
    type Int: Copy
        + TruncateToUsize
        + From<u8>
        + Not<Output = Self::Int>
        + Shl<u8, Output = Self::Int>
        + Sub<Output = Self::Int>
        + Add<Output = Self::Int>
        + Ord
        + BitAnd<Output = Self::Int>;
    const BITS: u8;
    fn to_int(self) -> Self::Int;
    fn from_int(n: Self::Int) -> Self;
}

impl Ip for Ipv4Addr {
    type Int = u32;
    const BITS: u8 = 32;
    fn to_int(self) -> Self::Int {
        u32::from(self)
    }
    fn from_int(n: Self::Int) -> Self {
        Ipv4Addr::from(n)
    }
}

impl Ip for Ipv6Addr {
    type Int = u128;
    const BITS: u8 = 128;
    fn to_int(self) -> Self::Int {
        u128::from(self)
    }
    fn from_int(n: Self::Int) -> Self {
        Ipv6Addr::from(n)
    }
}

fn available_ips<T: Ip>(used_addresses: HashSet<T>, prefix: u8) -> Vec<T> {
    assert!(prefix <= T::BITS, "Invalid prefix length");
    let first = used_addresses.iter().next().unwrap();
    let host_bits = T::BITS - prefix;
    let one = T::Int::from(1);
    let host_mask = (one << host_bits) - one;
    let network_address_int = first.to_int() & !host_mask;
    let range_size = one << host_bits;
    let capacity = range_size.truncate_to_usize() - used_addresses.len() - 1;
    let mut available = Vec::with_capacity(capacity);
    let mut i = one;
    while i < range_size {
        let address_int = network_address_int + i;
        let address = T::from_int(address_int);
        if !used_addresses.contains(&address) {
            available.push(address);
        }
        i = i + one;
    }
    available
}

fn main() -> Result<()> {
    let mut cmd = Cli::command();
    cmd.build();
    let args = Cli::from_arg_matches(&cmd.clone().get_matches())?;
    match args.command {
        Commands::Init { count } => {
            let path = Path::new(args.config.as_ref());
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
            if count.is_none() {
                fs::write(
                    path,
                    Meshs::new([Mesh::default()], 24, 120).to_json()?.as_bytes(),
                )?;
            } else {
                let count = count.unwrap();
                let ipv4_prefix = 32
                    - ((count + IPV4_NETWORK_BROADCAST_OVERHEAD) as f32)
                        .log2()
                        .ceil() as u8;
                let ipv6_prefix =
                    128 - ((count + RESERVED_IPV6_ADDRESS_COUNT) as f32).log2().ceil() as u8;
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
                        Some("place.holder.local.arpa:51820"),
                    ));
                }
                fs::write(
                    path,
                    Meshs::new(meshs, ipv4_prefix, ipv6_prefix)
                        .to_json()?
                        .as_bytes(),
                )?;
            }
        }
        Commands::Convert { output } => {
            let output = Path::new(output.as_ref());
            if output.is_file() {
                bail!("Output should not be file")
            } else if !output.exists() {
                bail!("Output directory does not exist")
            }
            let config_map = Conf::new(read_config(args.config.as_ref())?).create_all()?;
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
                fs::write(path, config.as_bytes())?;
            }
        }
        Commands::Append {
            tag,
            in_place,
            count,
        } => {
            let count = count.unwrap_or(1);
            let mut meshs = read_config(args.config.as_ref())?;
            let c = meshs.meshs.len() as u32 + count;
            if c > 16_777_214 {
                bail!("Total number of meshes exceed 16,777,214")
            }
            meshs.ipv4_prefix =
                32 - ((c + IPV4_NETWORK_BROADCAST_OVERHEAD) as f32).log2().ceil() as u8;
            meshs.ipv6_prefix =
                128 - ((c + RESERVED_IPV6_ADDRESS_COUNT) as f32).log2().ceil() as u8;
            let mut available_ipv4 = available_ips(
                meshs
                    .iter()
                    .map(|mesh| Ipv4Addr::from_str(&mesh.ipv4).unwrap())
                    .collect(),
                meshs.ipv4_prefix,
            )
            .into_iter();
            let mut available_ipv6 = available_ips(
                meshs
                    .iter()
                    .map(|mesh| Ipv6Addr::from_str(&mesh.ipv6).unwrap())
                    .collect(),
                meshs.ipv6_prefix,
            )
            .into_iter();
            let mut rng = rand::thread_rng();
            let mut meshs_vec = meshs.meshs.into_vec();
            for (i, _) in (meshs_vec.len() as u32..c).enumerate() {
                let i = i + 1;
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
                    available_ipv4.next().unwrap().to_string(),
                    available_ipv6.next().unwrap().to_string(),
                    Some("place.holder.local.arpa:51820"),
                ));
            }
            meshs.meshs = meshs_vec.into_boxed_slice();
            let json = meshs.to_json()?;
            if in_place {
                fs::write(args.config.as_ref(), json.as_bytes())?;
            } else {
                println!("{}", json);
            }
        }
    }
    Ok(())
}
