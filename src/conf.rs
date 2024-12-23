use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    net::{Ipv4Addr, Ipv6Addr},
    rc::Rc,
    str::FromStr,
};

use anyhow::{bail, format_err, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::mesh::{self, Mesh, Meshs};

#[derive(Default, Debug)]
pub struct Conf {
    meshs: Rc<Meshs>,
    verified: HashSet<Box<str>>,
}

impl Conf {
    pub fn create_single(&mut self, self_tag: impl AsRef<str>) -> Result<Box<str>> {
        let mut config = String::new();
        let meshs = self.meshs.clone();
        let self_mesh = meshs
            .iter()
            .find(|mesh| &*mesh.tag == self_tag.as_ref())
            .unwrap()
            .clone();
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
        if meshs.ipv4_prefix > 32 {
            bail!("The ipv4_prefix should not be greater than 32.")
        }
        if meshs.ipv6_prefix > 128 {
            bail!("The ipv6_prefix should not be greater than 128.")
        }
        for mesh in meshs.iter() {
            let self_tag = &mesh.tag;
            map.insert(self_tag.clone(), self.create_single(self_tag)?);
        }
        Ok(map)
    }

    fn verify(&mut self, mesh: &Mesh) -> Result<()> {
        if self.verified.contains(&mesh.tag) {
            return Ok(());
        }
        Ipv4Addr::from_str(&mesh.ipv4).map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        Ipv6Addr::from_str(&mesh.ipv6).map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        let prikey = STANDARD
            .decode(&*mesh.prikey)
            .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        if prikey.len() != 32 {
            bail!("[{}] The length of PrivateKey does not equal 32.", mesh.tag);
        };
        let pubkey = STANDARD
            .decode(&*mesh.pubkey)
            .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?;
        if pubkey.len() != 32 {
            bail!("[{}] The length of PublicKey does not equal 32.", mesh.tag);
        };
        let ppubkey = PublicKey::from(&StaticSecret::from(
            TryInto::<[u8; 32]>::try_into(prikey.as_slice())
                .map_err(|e| format_err!("[{}] {}", &mesh.tag, e))?,
        ));
        if *ppubkey.as_bytes() != *pubkey {
            bail!(
                "[{}] The PublicKey and PrivateKey do not form a pair.",
                mesh.tag
            );
        }
        if mesh.endpoint.contains('[') && mesh.endpoint.contains(']') {
            let i = mesh.endpoint.rfind(']').unwrap();
            mesh.endpoint[i..].rfind(':')
        } else {
            mesh.endpoint.rfind(':')
        }
        .context(format!("[{}] The endpoint does not have a port.", mesh.tag))?;
        self.verified.insert(mesh.tag.clone());
        Ok(())
    }
}
