use std::{
    collections::HashMap,
    fmt::{self, Write},
};

use crate::mesh::{Mesh, Meshs};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    FmtError(#[from] fmt::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[derive(Default, Debug)]
pub struct Conf {
    meshs: Meshs,
}

impl Conf {
    pub fn new(meshs: Meshs) -> Self {
        Conf { meshs }
    }

    pub fn create_single(&self, this_mesh: &Mesh) -> Result<Box<str>, Error> {
        let mut config = String::new();
        write!(
            config,
            "\
[Interface]
# PublicKey = {}
PrivateKey = {}
",
            this_mesh.key_pair.pubkey, this_mesh.key_pair.prikey,
        )?;
        if let Some(e) = &this_mesh.endpoint {
            write!(
                config,
                "\
ListenPort = {}
",
                e.split(':').last().unwrap(),
            )?;
        }
        write!(
            config,
            "\
Address = {}/{}
Address = {}/{}
",
            this_mesh.ipv4, self.meshs.ipv4_prefix, this_mesh.ipv6, self.meshs.ipv6_prefix
        )?;
        for mesh in self.meshs.iter() {
            if mesh == this_mesh {
                continue;
            }
            write!(
                config,
                "
[Peer]
PublicKey = {}
",
                mesh.key_pair.pubkey
            )?;
            if let Some(e) = &mesh.endpoint {
                write!(
                    config,
                    "\
Endpoint = {}
",
                    e
                )?;
            }
            write!(
                config,
                "\
AllowedIPs = {}/32, {}/128
",
                mesh.ipv4, mesh.ipv6
            )?;
        }
        Ok(config.into())
    }

    pub fn create_all(&self) -> Result<HashMap<Box<str>, Box<str>>, Error> {
        let mut config_map = HashMap::new();
        let mut tag_counts: HashMap<_, usize> = HashMap::new();
        for mesh in self.meshs.iter() {
            let this_tag = &mesh.tag;
            *tag_counts.entry(this_tag).or_insert(0) += 1;
            config_map.insert(this_tag.clone(), self.create_single(mesh)?);
        }
        let duplicates: Box<[_]> = tag_counts.iter().filter(|(_, &count)| count > 1).collect();
        if !duplicates.is_empty() {
            const WARN: &str = "\x1b[0;33mWARNING\x1b[0m";
            for (tag, _) in duplicates {
                eprintln!("{}: Multiple meshes with the same tag: {:?}", WARN, tag);
            }
            eprintln!(
                "{}: This will occur some overwrite, it should be avoid",
                WARN
            );
        }
        Ok(config_map)
    }
}
