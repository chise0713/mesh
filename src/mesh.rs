use std::{fs::File, io::Read};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Mesh {
    pub tag: Box<str>,
    pub pubkey: Box<str>,
    pub prikey: Box<str>,
    pub ipv4: Box<str>,
    pub ipv6: Box<str>,
}

impl Mesh {
    pub fn new(
        tag: impl Into<Box<str>>,
        pubkey: impl Into<Box<str>>,
        prikey: impl Into<Box<str>>,
        ipv4: impl Into<Box<str>>,
        ipv6: impl Into<Box<str>>,
    ) -> Self {
        Mesh {
            tag: tag.into(),
            pubkey: pubkey.into(),
            prikey: prikey.into(),
            ipv4: ipv4.into(),
            ipv6: ipv6.into(),
        }
    }
    pub fn to_json(&self) -> Box<str> {
        serde_json::to_string_pretty(self).unwrap().into_boxed_str()
    }
    pub fn from_json(v: impl AsRef<str>) -> Self {
        serde_json::from_str(v.as_ref()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Meshs {
    pub meshs: Box<[Mesh]>,
    pub ipv4_prefix: u8,
    pub ipv6_prefix: u8,
}

impl Meshs {
    pub fn new(meshs: impl Into<Box<[Mesh]>>, ipv4_prefix: u8, ipv6_prefix: u8) -> Self {
        Meshs {
            meshs: meshs.into(),
            ipv4_prefix,
            ipv6_prefix,
        }
    }
    pub fn to_json(&self) -> Box<str> {
        serde_json::to_string_pretty(self).unwrap().into_boxed_str()
    }
    pub fn from_json(v: impl AsRef<str>) -> Self {
        serde_json::from_str(v.as_ref()).unwrap()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Mesh> {
        self.meshs.iter()
    }
}

impl IntoIterator for Meshs {
    type Item = Mesh;
    type IntoIter = <Box<[Mesh]> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.meshs.to_vec().into_iter()
    }
}

pub fn read_file<T: for<'de> Deserialize<'de>>(path: impl AsRef<str>) -> Result<T> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: T = serde_json::from_str(&contents)?;
    Ok(data)
}
