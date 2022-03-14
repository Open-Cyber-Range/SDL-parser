use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize, Deserializer};
use bytesize::ByteSize;

fn parse_bytesize<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Ok(s.parse::<ByteSize>().map_err(|_| serde::de::Error::custom("Failed"))?.0 as u32)
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
  VM
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
  pub package: Option<Package>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Package {
  pub name: String,
  pub version: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "type")]
    pub type_field: NodeType,
    pub description: Option<String>,
    pub template: String,
    pub flavor: Flavor,
    pub source: Option<Source>
}

pub type NodeMap = HashMap<String, Node>;

#[cfg(test)]
mod tests {
  use super::*;

  
  #[test]
  fn includes_node_requirements_with_source_template() {
    let node_sdl = r#"
        win10:
            type: VM
            template: windows10
            flavor:
                ram: 4gb
                cpu: 4
            source:
                template: windows10-template
        "#;
    let parsed_nodes = serde_yaml::from_str::<NodeMap>(node_sdl).unwrap();

    let name = (*parsed_nodes.keys().collect::<Vec<&String>>().get(0).unwrap()).clone();
    let node = (*parsed_nodes.values().collect::<Vec<&Node>>().get(0).unwrap()).clone();
    
    assert_eq!(name, "win10".to_string());
    assert_eq!(node.template, "windows10");
    assert_eq!(node.flavor.ram, 4000000000);
    assert_eq!(node.flavor.cpu, 4);
    assert_eq!(node.description, None);
    assert_eq!(node.source.unwrap().template.unwrap(), "windows10-template");
  }

   #[test]
  fn includes_all_node_requirements_with_source_package() {
    let node_sdl = r#"
        win10:
            type: VM
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
    let parsed_nodes = serde_yaml::from_str::<NodeMap>(node_sdl).unwrap();

    let name = (*parsed_nodes.keys().collect::<Vec<&String>>().get(0).unwrap()).clone();
    let node = (*parsed_nodes.values().collect::<Vec<&Node>>().get(0).unwrap()).clone();
    
    assert_eq!(name, "win10".to_string());
    assert_eq!(node.template, "windows10");
    assert_eq!(node.flavor.ram, 4000000000);
    assert_eq!(node.flavor.cpu, 4);
    assert_eq!(node.description.unwrap(), "win10 node for OCR");
    assert_eq!(node.source.clone().unwrap().package.unwrap().name, "basic-windows10");
    assert_eq!(node.source.unwrap().package.unwrap().version, "*");
  }


  #[test]
  fn includes_minimal_node_requirements() {
    let node_sdl = r#"
        win10:
            type: VM
            template: windows10
            flavor:
                ram: 4gb
                cpu: 2
        "#;
    let parsed_nodes = serde_yaml::from_str::<NodeMap>(node_sdl).unwrap();

    let name = (*parsed_nodes.keys().collect::<Vec<&String>>().get(0).unwrap()).clone();
    let node = (*parsed_nodes.values().collect::<Vec<&Node>>().get(0).unwrap()).clone();
    
    assert_eq!(name, "win10".to_string());
    assert_eq!(node.template, "windows10");
    assert_eq!(node.flavor.ram, 4000000000);
    assert_eq!(node.flavor.cpu, 2);
    assert_eq!(node.description, None);
  }
}