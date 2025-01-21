use std::{
    cell::RefCell,
    fmt,
    fs::File,
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{de, Deserialize, Deserializer, Serialize};

macro_rules! create_boxed_struct {
    ($($struct_name:ident),+) => {
        $(
            #[derive(Serialize, Debug, Default, PartialEq, Eq, Clone)]
            pub struct $struct_name(Box<str>);
            impl Deref for $struct_name {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl DerefMut for $struct_name {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
            impl fmt::Display for $struct_name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
        )+
    };
}

macro_rules! impl_ip_deserialize {
    ($type:ty, $parse_fn:path) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s: Box<str> = Deserialize::deserialize(deserializer)?;
                if $parse_fn(&s).is_ok() {
                    Ok(Self(s))
                } else {
                    Err(de::Error::custom("Invalid IP address"))
                }
            }
        }
    };
}

macro_rules! impl_base64_deserialize {
    ($($type:ty),+) => {
        $(
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s: Box<str> = Deserialize::deserialize(deserializer)?;
                let bytes = STANDARD.decode(&*s).map_err(de::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(de::Error::custom("Invalid length of decoded key"))
                }
                Ok(Self(s))
            }
        }
        )+
    };
}

create_boxed_struct!(
    PublicKeyBoxStr,
    PrivateKeyBoxStr,
    Ipv4BoxStr,
    Ipv6BoxStr,
    EndpointBoxStr
);

#[derive(Debug, thiserror::Error)]
pub enum EndpointParseError {
    #[error("Invalid endpoint address syntax")]
    InvalidSyntax,

    #[error("Missing port in endpoint")]
    MissingPort,
}

impl<'de> Deserialize<'de> for EndpointBoxStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Box<str> = Deserialize::deserialize(deserializer)?;
        let validate = || {
            if s.contains('[') && s.contains(']') {
                let i = s.rfind(']').unwrap();
                if s[i..].rfind(':').is_none() {
                    return Err(EndpointParseError::MissingPort);
                }
            } else if s.contains('[') || s.contains(']') {
                return Err(EndpointParseError::InvalidSyntax);
            } else if s.rfind(':').is_none() {
                return Err(EndpointParseError::MissingPort);
            }
            Ok(())
        };

        validate().map_err(de::Error::custom)?;
        Ok(EndpointBoxStr(s))
    }
}
impl_base64_deserialize!(PublicKeyBoxStr, PrivateKeyBoxStr);
impl_ip_deserialize!(Ipv4BoxStr, Ipv4Addr::from_str);
impl_ip_deserialize!(Ipv6BoxStr, Ipv6Addr::from_str);

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Mesh {
    pub tag: Box<str>,
    pub pubkey: PublicKeyBoxStr,
    pub prikey: PrivateKeyBoxStr,
    pub ipv4: Ipv4BoxStr,
    pub ipv6: Ipv6BoxStr,
    pub endpoint: EndpointBoxStr,
    #[serde(skip)]
    pub(crate) unique_id: RefCell<u16>,
}

impl Mesh {
    pub fn new(
        tag: impl Into<Box<str>>,
        pubkey: impl Into<Box<str>>,
        prikey: impl Into<Box<str>>,
        ipv4: impl Into<Box<str>>,
        ipv6: impl Into<Box<str>>,
        endpoint: impl Into<Box<str>>,
    ) -> Self {
        Mesh {
            tag: tag.into(),
            pubkey: PublicKeyBoxStr(pubkey.into()),
            prikey: PrivateKeyBoxStr(prikey.into()),
            ipv4: Ipv4BoxStr(ipv4.into()),
            ipv6: Ipv6BoxStr(ipv6.into()),
            endpoint: EndpointBoxStr(endpoint.into()),
            unique_id: 0.into(),
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
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Mesh> {
        self.meshs.iter_mut()
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
