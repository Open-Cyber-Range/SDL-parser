use anyhow::Result;
use bytesize::ByteSize;
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

fn parse_bytesize<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<ByteSize>()
        .map_err(|_| serde::de::Error::custom("Failed"))?
        .0 as u32)
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    VM,
    Network,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Direction {
    Ingress,
    Egress,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Flavor {
    #[serde(deserialize_with = "parse_bytesize")]
    pub ram: u32,
    pub cpu: u32,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub template: Option<String>,
    #[serde(default, deserialize_with = "deserialize_struct_case_insensitive")]
    pub package: Option<Package>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    #[serde(rename = "type")]
    pub type_field: String,
    pub cidr: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    pub rule: Rule,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub direction: Direction,
    pub description: String,
    pub allowed_address: Option<Vec<String>>,
    pub port: u16,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub type_field: NodeType,
    #[serde(alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(alias = "Address", alias = "ADDRESS")]
    #[serde(default, deserialize_with = "deserialize_struct_case_insensitive")]
    pub address: Option<Address>,
    #[serde(alias = "Policy", alias = "POLICY")]
    #[serde(default, deserialize_with = "deserialize_struct_case_insensitive")]
    pub policy: Option<Policy>,
    #[serde(alias = "Flavor", alias = "FLAVOR")]
    #[serde(default, deserialize_with = "deserialize_struct_case_insensitive")]
    pub flavor: Option<Flavor>,
    #[serde(alias = "Source", alias = "SOURCE")]
    #[serde(default, deserialize_with = "deserialize_struct_case_insensitive")]
    pub source: Option<Source>,
}

pub type NodeMap = HashMap<String, Node>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_node_requirements_with_network_type() {
        let node_sdl = r#"
            type: Network
            dependencies:
                - 1
                - kolm
                - serde
            description: a network
            address:
                type: ipv4
                cidr: 10.10.10.0/24
            policy:
                type: network
                rule:
                    direction: Ingress
                    description: a-description
                    allowed_address:
                        - some-ip
                        - some-address
                        - some-number-5
                    port: 8080
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn includes_node_requirements_with_source_template() {
        let node_sdl = r#"
            type: VM
            template: windows10
            flavor:
                ram: 4gb
                cpu: 4
            source:
                template: windows10-template
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();

        assert_eq!(node.source.unwrap().template.unwrap(), "windows10-template");
    }

    #[test]
    fn includes_all_node_requirements_with_source_package() {
        let node_sdl = r#"
            type: VM
            dependencies: [pub-net]
            template: windows10
            description: win10 node for OCR
            flavor:
                ram: 4gb
                cpu: 4
            source:
                package:
                    name: basic-windows10
                    version: '*'
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        assert_eq!(node.description.unwrap(), "win10 node for OCR");
        assert_eq!(
            node.source.clone().unwrap().package.unwrap().name,
            "basic-windows10"
        );
        assert_eq!(node.source.unwrap().package.unwrap().version, "*");
    }

    #[test]
    fn includes_minimal_node_requirements() {
        let node_sdl = r#"
            type: VM
            template: windows10
            flavor:
                ram: 4gb
                cpu: 2
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        let flavor = node.flavor.unwrap();
        assert_eq!(flavor.ram, 4000000000);
        assert_eq!(flavor.cpu, 2);
        assert_eq!(node.description, None);
    }

    #[test]
    fn fields_are_case_insensitive() {
        let node_sdl = r#"
            Type: Network
            Dependencies:
                - 1
                - kolm
                - serde
            Description: a network
            Address:
                TYPE: ipv4
                Cidr: 10.10.10.0/24
            POLICY:
                Type: network
                Rule:
                    Direction: Ingress
                    Description: a-description
                    Allowed_address:
                        - some-ip
                        - some-address
                        - some-number-5
                    Port: 8080

            Template: windows10
            Flavor:
                Ram: 4gb
                Cpu: 4
            Source:
                Template: windows10-template
                Package:
                    Name: basic-windows10
                    Version: '*'
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        insta::assert_debug_snapshot!(node);
    }
}
