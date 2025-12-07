use std::{
    collections::HashMap,
    fmt::{self, Write},
};

use crate::mesh::{Mesh, Meshs};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("duplicate tags: {}", DisplayTags(.0))]
    DuplicateTags(Box<[Box<str>]>),
}

struct DisplayTags<'a>(&'a [Box<str>]);

impl<'a> std::fmt::Display for DisplayTags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for (i, tag) in self.0.iter().enumerate() {
            write!(f, "\"{}\"", tag)?;
            if i + 1 != self.0.len() {
                write!(f, ", ")?;
            }
        }
        write!(f, " ]")
    }
}

#[derive(Default, Debug)]
pub struct Conf {
    pub meshs: Meshs,
}

impl Conf {
    pub fn new(meshs: Meshs) -> Self {
        Conf { meshs }
    }

    pub fn create_single(&self, this_mesh: &Mesh) -> Result<Box<str>, Error> {
        let mut config = String::new();
        writeln!(
            config,
            "\
[Interface]
# PublicKey = {}
PrivateKey = {}",
            this_mesh.key_pair.pubkey, this_mesh.key_pair.prikey,
        )?;
        if let Some(e) = &this_mesh.endpoint {
            writeln!(
                config,
                "\
ListenPort = {}",
                e.split(':').last().unwrap(),
            )?;
        }
        writeln!(
            config,
            "\
Address = {}/{}
Address = {}/{}",
            this_mesh.ipv4, self.meshs.ipv4_prefix, this_mesh.ipv6, self.meshs.ipv6_prefix
        )?;
        for mesh in self.meshs.iter() {
            if mesh == this_mesh {
                continue;
            }
            writeln!(
                config,
                "
[Peer]
PublicKey = {}",
                mesh.key_pair.pubkey
            )?;
            if let Some(e) = &mesh.endpoint {
                writeln!(
                    config,
                    "\
Endpoint = {}",
                    e
                )?;
            }
            writeln!(
                config,
                "\
AllowedIPs = {}/32, {}/128",
                mesh.ipv4, mesh.ipv6
            )?;
        }
        Ok(config.into())
    }

    pub fn create_all(&self) -> Result<HashMap<Box<str>, Box<str>>, Error> {
        let mut config_map = HashMap::new();
        let mut tag_counts: HashMap<_, usize> = HashMap::new();
        for mesh in self.meshs.iter() {
            let this_tag = mesh.tag.clone();
            *tag_counts.entry(this_tag.clone()).or_insert(0) += 1;
            config_map.insert(this_tag, self.create_single(mesh)?);
        }
        let duplicates: Box<[_]> = tag_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(tag, _)| tag)
            .collect();
        if duplicates.is_empty() {
            Ok(config_map)
        } else {
            Err(Error::DuplicateTags(duplicates))
        }
    }
}
