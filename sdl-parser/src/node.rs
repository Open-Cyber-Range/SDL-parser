use anyhow::Result;
use bytesize::ByteSize;
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

fn parse_bytesize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<ByteSize>()
        .map_err(|_| serde::de::Error::custom("Failed"))?
        .0 as u64)
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    VM,
    Network,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Resources {
    #[serde(deserialize_with = "parse_bytesize")]
    pub ram: u64,
    pub cpu: u32,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SourceArray {
    Source(Source),
    ShortSource(String),
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub name: String,
    pub version: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub type_field: NodeType,
    #[serde(default, alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(
        default,
        alias = "Resources",
        alias = "RESOURCES",
        deserialize_with = "deserialize_struct_case_insensitive"
    )]
    pub resources: Option<Resources>,
    #[serde(
        default,
        rename = "source",
        alias = "Source",
        alias = "SOURCE",
        skip_serializing
    )]
    _source_helper: Option<SourceArray>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
}

impl Node {
    pub fn map_source(&mut self) {
        match &mut self._source_helper.take() {
            Some(SourceArray::Source(source)) => {
                self.source = Some(source.to_owned());
            }
            Some(SourceArray::ShortSource(source)) => {
                self.source = Some(Source {
                    name: source.to_owned(),
                    version: "*".to_string(),
                });
            }
            None => {}
        }
    }
}

pub type NodeMap = HashMap<String, Node>;

#[cfg(test)]
mod tests {
    use crate::parse_sdl;

    use super::*;

    #[test]
    fn source_fields_are_mapped_correctly() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                deb-10:
                    type: VM
                    source:
                        name: debian10
                        version: '*'
        "#;
        let nodes = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }

    #[test]
    fn node_source_longhand_is_parsed() {
        let longhand_source = r#"
            type: VM
            source: 
                name: package-name
                version: 1.2.3
                    
        "#;
        let node = serde_yaml::from_str::<Node>(longhand_source).unwrap();
        insta::assert_debug_snapshot!(node);
    }
    #[test]
    fn node_source_shorthand_is_parsed() {
        let shorthand_source = r#"
            type: VM
            source: package-name
                    
        "#;
        let node = serde_yaml::from_str::<Node>(shorthand_source).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn includes_node_requirements_with_network_type() {
        let node_sdl = r#"
            type: Network
            description: a network
        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn includes_node_requirements_with_source_template() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources:
                        ram: 2 gib
                        cpu: 2
                    source: windows10
        "#;
        let nodes = parse_sdl(sdl).unwrap().scenario.nodes.unwrap();
        insta::assert_debug_snapshot!(nodes);
    }
}
