#![cfg(test)]

use crate::mesh::{
    EndpointBoxStr, FromJson as _, Ipv4BoxStr, Ipv6BoxStr, Mesh, Meshs, ToJson as _,
};

#[test]
fn test_eq() {
    let mut mesh = Mesh::new(
        "1",
        "L+V9o0fNYkMVKNqsX7spBzD/9oSvxM/C7ZCZX1jLO3Q=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "10.0.0.1",
        "fd00::1",
        Some("test.local.arpa:51820"),
    );
    let meshs_orig = Meshs::new([mesh.clone()], 24, 120);
    let json = meshs_orig.to_json().unwrap();
    let meshs_de = Meshs::from_json(json).unwrap();
    assert_eq!(meshs_orig, meshs_de);
    mesh.endpoint = None;
    let meshs_orig = Meshs::new([mesh], 24, 120);
    let json = meshs_orig.to_json().unwrap();
    let meshs_de = Meshs::from_json(json).unwrap();
    assert_eq!(meshs_orig, meshs_de);
}

#[test]
fn test_de() {
    let mut mesh = Mesh::new(
        "1",
        "L+V9o0fNYkMVKNqsX7spBzD/9oSvxM/C7ZCZX1jLO3Q=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "10.0.0.1",
        "fd00::1",
        Some("test.local.arpa:51820"),
    );
    let mut test_fields: [(&str, Box<str>); 5] = [
        ("pubkey", Box::from("")),
        ("prikey", Box::from("")),
        ("ipv4", Box::from("invalid-ip")),
        ("ipv6", Box::from("invalid-ipv6")),
        ("endpoint", Box::from("invalid-endpoint")),
    ];
    let original_values = [
        mesh.key_pair.pubkey.clone(),
        mesh.key_pair.prikey.clone(),
        mesh.ipv4.0.clone(),
        mesh.ipv6.0.clone(),
        mesh.endpoint.clone().unwrap().0,
    ];
    for (field, value) in test_fields.iter_mut() {
        match *field {
            "pubkey" => mesh.key_pair.pubkey = value.clone(),
            "prikey" => mesh.key_pair.prikey = value.clone(),
            "ipv4" => mesh.ipv4 = Ipv4BoxStr(value.clone()),
            "ipv6" => mesh.ipv6 = Ipv6BoxStr(value.clone()),
            "endpoint" => mesh.endpoint = Some(EndpointBoxStr(value.clone())),
            _ => unreachable!(),
        }
        Mesh::from_json(mesh.to_json().unwrap()).unwrap_err();
        match *field {
            "pubkey" => mesh.key_pair.pubkey = original_values[0].clone(),
            "prikey" => mesh.key_pair.prikey = original_values[1].clone(),
            "ipv4" => mesh.ipv4 = Ipv4BoxStr(original_values[2].clone()),
            "ipv6" => mesh.ipv6 = Ipv6BoxStr(original_values[3].clone()),
            "endpoint" => mesh.endpoint = Some(EndpointBoxStr(original_values[4].clone())),
            _ => unreachable!(),
        }
    }
    mesh.key_pair.prikey = "y3f0fu/krxHKNdt86ElVqBs9jLdvn4AYncjlBKWe/nA=".into();
    Mesh::from_json(mesh.to_json().unwrap()).unwrap_err();
    mesh.key_pair.prikey = original_values[0].clone();
    mesh.key_pair.prikey = "y3f0fu/krxHKNdt86ElVqBs9jLdvn4AYncjlBKWe/nA=".into();
    Mesh::from_json(mesh.to_json().unwrap()).unwrap_err();
    mesh.key_pair.prikey = original_values[1].clone();
    Meshs::from_json(Meshs::new([mesh], 33, 129).to_json().unwrap()).unwrap_err();
}
