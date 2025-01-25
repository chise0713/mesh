use std::{
    fmt,
    fs::File,
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use x25519_dalek::{PublicKey, StaticSecret};

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
                $parse_fn(&s).map_err(de::Error::custom)?;
                Ok(Self(s))
            }
        }
    };
}

pub trait ToJson {
    fn to_json(&self) -> Result<Box<str>, serde_json::Error>;
}

pub trait FromJson {
    fn from_json(v: impl AsRef<str>) -> Result<Self, serde_json::Error>
    where
        Self: Sized;
}

macro_rules! impl_json {
    ($($type:ident),+) => {
        $(
            impl ToJson for $type {
            fn to_json(&self) -> Result<Box<str>, serde_json::Error> {
                    serde_json::to_string_pretty(self).map(|s| s.into_boxed_str())
                }
            }
            impl FromJson for $type {
                fn from_json(v: impl AsRef<str>) -> Result<Self,serde_json::Error> {
                    serde_json::from_str(v.as_ref())
                }
            }
        )+
    };
}

create_boxed_struct!(Ipv4BoxStr, Ipv6BoxStr, EndpointBoxStr);

#[derive(Serialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct KeyPair {
    pub pubkey: Box<str>,
    pub prikey: Box<str>,
}

impl<'de> Deserialize<'de> for KeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELD_PUBKEY: &str = "pubkey";
        const FIELD_PRIKEY: &str = "prikey";
        struct KeyPairVisitor;
        impl<'de> Visitor<'de> for KeyPairVisitor {
            type Value = KeyPair;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with pubkey and prikey fields")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut pubkey_str: Option<&str> = None;
                let mut prikey_str: Option<&str> = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key.as_ref() {
                        FIELD_PUBKEY => {
                            if pubkey_str.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_PUBKEY));
                            }
                            pubkey_str = Some(map.next_value()?);
                        }
                        FIELD_PRIKEY => {
                            if prikey_str.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_PRIKEY));
                            }
                            prikey_str = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let pubkey_str =
                    pubkey_str.ok_or_else(|| de::Error::missing_field(FIELD_PUBKEY))?;
                let prikey_str =
                    prikey_str.ok_or_else(|| de::Error::missing_field(FIELD_PRIKEY))?;
                let prikey = STANDARD
                    .decode(&*prikey_str)
                    .map_err(|e| de::Error::custom(format!("Failed to decode prikey: {}", e)))?;

                let pubkey = STANDARD
                    .decode(&*pubkey_str)
                    .map_err(|e| de::Error::custom(format!("Failed to decode pubkey: {}", e)))?;

                let ppubkey = PublicKey::from(&StaticSecret::from(
                    <[u8; 32]>::try_from(prikey.as_slice()).map_err(de::Error::custom)?,
                ));
                if *ppubkey.as_bytes() != *pubkey {
                    return Err(de::Error::custom("Key pair mismatch"));
                }
                Ok(KeyPair {
                    pubkey: pubkey_str.into(),
                    prikey: prikey_str.into(),
                })
            }
        }
        deserializer.deserialize_map(KeyPairVisitor)
    }
}

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
impl_ip_deserialize!(Ipv4BoxStr, Ipv4Addr::from_str);
impl_ip_deserialize!(Ipv6BoxStr, Ipv6Addr::from_str);

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Mesh {
    pub tag: Box<str>,
    #[serde(flatten)]
    pub key_pair: KeyPair,
    pub ipv4: Ipv4BoxStr,
    pub ipv6: Ipv6BoxStr,
    pub endpoint: EndpointBoxStr,
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
            key_pair: KeyPair {
                pubkey: pubkey.into(),
                prikey: prikey.into(),
            },
            ipv4: Ipv4BoxStr(ipv4.into()),
            ipv6: Ipv6BoxStr(ipv6.into()),
            endpoint: EndpointBoxStr(endpoint.into()),
        }
    }
}

fn deserialize_with_max<'de, const MAX: u8, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    if value > MAX {
        Err(de::Error::custom(format!(
            "Invalid value: {} (maximum {})",
            value, MAX
        )))
    } else {
        Ok(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Meshs {
    pub meshs: Box<[Mesh]>,
    #[serde(deserialize_with = "deserialize_with_max::<32, _>")]
    pub ipv4_prefix: u8,
    #[serde(deserialize_with = "deserialize_with_max::<128, _>")]
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
    pub fn iter(&self) -> impl Iterator<Item = &Mesh> {
        self.meshs.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Mesh> {
        self.meshs.iter_mut()
    }
}

impl_json!(Mesh, Meshs);

impl IntoIterator for Meshs {
    type Item = Mesh;
    type IntoIter = <Box<[Mesh]> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.meshs.to_vec().into_iter()
    }
}

pub fn read_file<T: FromJson>(path: impl AsRef<str>) -> Result<T> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data = T::from_json(&contents)?;
    Ok(data)
}
