use std::{
    collections::HashMap,
    fmt::Write,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::mesh::{self, Meshs};

pub fn create_single_config(meshs: &Meshs, self_tag: impl AsRef<str>) -> Result<Box<str>> {
    let mut config = String::new();
    let self_mesh = meshs
        .iter()
        .find(|mesh| mesh.tag.as_ref() == self_tag.as_ref())
        .unwrap();
    write!(
        config,
        "\
[Interface]
# PublicKey = {}
PrivateKey = {}
ListenPort = {}
Address = {}/{}
Address = {}/{}
",
        self_mesh.pubkey,
        self_mesh.prikey,
        self_mesh.endpoint.split(':').last().unwrap(),
        self_mesh.ipv4,
        &meshs.ipv4_prefix,
        self_mesh.ipv6,
        &meshs.ipv6_prefix
    )?;
    for mesh in meshs.iter() {
        if mesh == self_mesh {
            continue;
        }
        write!(
            config,
            "
[Peer]
PublicKey = {}
Endpoint = {}
AllowedIPs = {}/32, {}/128
",
            mesh.pubkey, mesh.endpoint, mesh.ipv4, mesh.ipv6
        )?;
    }
    return Ok(config.into());
}

pub fn create_all_config(path: impl AsRef<str>) -> Result<HashMap<Box<str>, Box<str>>> {
    let mut map = HashMap::new();
    let meshs: Meshs = mesh::read_file(path)?;
    verify(&meshs)?;
    for mesh in meshs.iter() {
        let self_tag = &mesh.tag;
        map.insert(self_tag.clone(), create_single_config(&meshs, self_tag)?);
    }
    Ok(map)
}

fn verify(meshs: &Meshs) -> Result<()> {
    for mesh in meshs.iter() {
        Ipv4Addr::from_str(&mesh.ipv4)?;
        Ipv6Addr::from_str(&mesh.ipv6)?;
        let prikey = STANDARD.decode(&*mesh.prikey)?;
        if prikey.len() != 32 {
            bail!("The length of PrivateKey does not equal 32.");
        };
        let pubkey = STANDARD.decode(&*mesh.pubkey)?;
        if pubkey.len() != 32 {
            bail!("The length of PublicKey does not equal 32.");
        };
        let ppubkey = PublicKey::from(&StaticSecret::from(TryInto::<[u8; 32]>::try_into(
            prikey.as_slice(),
        )?));
        if *ppubkey.as_bytes() != *pubkey {
            bail!("The PublicKey and PrivateKey do not form a pair.");
        }
        if mesh.endpoint.contains('[') && mesh.endpoint.contains(']') {
            let i = mesh.endpoint.rfind(']').unwrap();
            mesh.endpoint[i..].rfind(':')
        } else {
            mesh.endpoint.rfind(':')
        }
        .context("The endpoint does not have a port.")?;
    }
    Ok(())
}
