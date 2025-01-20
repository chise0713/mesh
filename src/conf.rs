use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    rc::Rc,
};

use anyhow::{bail, format_err, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    mesh::{self, Mesh, Meshs},
    INTERNAL_CHECK_MESHS,
};

#[derive(Default, Debug)]
pub struct Conf {
    pub meshs: Rc<Meshs>,
    verified: HashSet<u16>,
}

impl Conf {
    pub fn create_single(&mut self, self_tag: impl AsRef<str>) -> Result<Box<str>> {
        let meshs = self.meshs.clone();
        if self.verified.is_empty() {
            let mut current_id = 1;
            meshs.iter().for_each(|item| {
                *item.unique_id.borrow_mut() = current_id;
                current_id += 1;
            });
        }
        let mut config = String::new();
        let self_mesh = meshs
            .iter()
            .find(|mesh| mesh.tag.as_ref() == self_tag.as_ref())
            .unwrap()
            .clone();
        if !self.verified.contains(&INTERNAL_CHECK_MESHS) {
            if meshs.ipv4_prefix > 32 {
                bail!("The ipv4_prefix should not be greater than 32")
            }
            if meshs.ipv6_prefix > 128 {
                bail!("The ipv6_prefix should not be greater than 128")
            }
            self.verified.insert(INTERNAL_CHECK_MESHS);
        }
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
            self.verify(&mesh)?;
            if *mesh == self_mesh {
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

    pub fn create_all(&mut self, path: impl AsRef<str>) -> Result<HashMap<Box<str>, Box<str>>> {
        let mut map = HashMap::new();
        self.meshs = Rc::new(mesh::read_file(path)?);
        let meshs = self.meshs.clone();
        let mut tag_counts: HashMap<_, usize> = HashMap::new();
        for mesh in meshs.iter() {
            let self_tag = &mesh.tag;
            *tag_counts.entry(self_tag.clone()).or_insert(0) += 1;
            map.insert(self_tag.clone(), self.create_single(self_tag)?);
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

    fn verify(&mut self, mesh: &Mesh) -> Result<()> {
        let unique_id = *mesh.unique_id.borrow();
        if self.verified.contains(&unique_id) {
            return Ok(());
        } else {
            self.verified.insert(unique_id);
        }
        let prikey = STANDARD
            .decode(&*mesh.prikey)
            .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        let pubkey = STANDARD
            .decode(&*mesh.pubkey)
            .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        let ppubkey = PublicKey::from(&StaticSecret::from(
            <[u8; 32]>::try_from(prikey.as_slice())
                .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?,
        ));
        if *ppubkey.as_bytes() != *pubkey {
            bail!(
                "[{}] The PublicKey and PrivateKey do not form a pair",
                mesh.tag
            );
        }
        Ok(())
    }
}
