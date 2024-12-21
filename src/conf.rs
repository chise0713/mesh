use std::{collections::HashMap, fmt::Write};

use anyhow::Result;

use crate::mesh::{self, Meshs};

pub fn create_single_config(meshs: &Meshs, self_tag: impl AsRef<str>) -> Result<Box<str>> {
    let mut config = String::new();
    let self_mesh = meshs
        .iter()
        .find(|mesh| mesh.tag.as_ref() == self_tag.as_ref())
        .unwrap();
    writeln!(
        config,
        "\
[Interface]
# PublicKey = {}
PrivateKey = {}
Address = {}/{}
Address = {}/{}
",
        self_mesh.pubkey,
        self_mesh.prikey,
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
AllowedIPs = {}/32, {}/128
",
            mesh.pubkey, mesh.ipv4, mesh.ipv6
        )?;
    }
    return Ok(config.into());
}

pub fn create_all_config(path: impl AsRef<str>) -> Result<HashMap<Box<str>, Box<str>>> {
    let mut map = HashMap::new();
    let meshs: Meshs = mesh::read_file(path)?;
    for mesh in meshs.iter() {
        let self_tag = &mesh.tag;
        map.insert(self_tag.clone(), create_single_config(&meshs, self_tag)?);
    }
    Ok(map)
}
