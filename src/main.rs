mod cli;

use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::{CommandFactory, FromArgMatches as _};
use cli::{Cli, Commands};
use mesh::{
    conf::Conf,
    mesh::{Mesh, Meshs},
};
use x25519_dalek::{PublicKey, StaticSecret};

fn main() -> Result<()> {
    let mut cmd = Cli::command();
    cmd.build();
    let args = Cli::from_arg_matches(&cmd.clone().get_matches())?;
    match args.command {
        Commands::Init { count } => {
            let path = PathBuf::from(&*args.config);
            if path.exists() {
                eprintln!("Config file already exsits.");
                eprint!("continue? [y/N]");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input.len() > 1 {
                    bail!("Invalid input.")
                }
                if !input.to_ascii_lowercase().contains('y') {
                    bail!("Aborted.")
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
                if count > 254 {
                    bail!("Convert count should not be greater than 254.");
                }
                let mut meshs = Vec::new();
                for i in 1..=count {
                    let secret = StaticSecret::random_from_rng(&mut rand::thread_rng());
                    let public = PublicKey::from(&secret);
                    let public = STANDARD.encode(public);
                    let secret = STANDARD.encode(secret);
                    meshs.push(Mesh::new(
                        i.to_string(),
                        public,
                        secret,
                        format!("10.0.0.{}", i),
                        format!("fd00::{:x}", i),
                        "place.holder.local.arpa:51820",
                    ));
                }
                write!(file, "{}", Meshs::new(meshs, 24, 120).to_json())?;
            }
        }
        Commands::Convert { output } => {
            let output = PathBuf::from(&*output);
            if output.is_file() {
                bail!("Output should not be file.")
            } else if !output.exists() {
                bail!("Output directory does not exist.")
            }
            let mut config = Conf::default();
            let config_map = config.create_all(args.config)?;
            for (tag, config) in config_map {
                if tag.is_empty() {
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
