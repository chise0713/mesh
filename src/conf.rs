use std::{collections::HashMap, fmt::Write, rc::Rc};

use anyhow::Result;

use crate::mesh::{self, Mesh, Meshs};

#[derive(Default, Debug)]
pub struct Conf {
    pub meshs: Rc<Meshs>,
}

impl Conf {
    pub fn create_single(&mut self, self_mesh: &Mesh) -> Result<Box<str>> {
        let meshs = self.meshs.clone();
        let mut config = String::new();
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
            self_mesh.key_pair.pubkey,
            self_mesh.key_pair.prikey,
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
                mesh.key_pair.pubkey, mesh.endpoint, mesh.ipv4, mesh.ipv6
            )?;
        }
        return Ok(config.into());
    }

    pub fn create_all(&mut self, path: impl AsRef<str>) -> Result<HashMap<Box<str>, Box<str>>> {
        let mut map = HashMap::new();
        self.meshs = Rc::new(mesh::read_file(path)?);
        let meshs = self.meshs.clone();
        let mut tag_counts: HashMap<_, usize> = HashMap::new();
        for mesh in meshs.iter() {
            let self_tag = &mesh.tag;
            *tag_counts.entry(self_tag).or_insert(0) += 1;
            map.insert(self_tag.clone(), self.create_single(mesh)?);
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
        Ok(map)
    }
}
