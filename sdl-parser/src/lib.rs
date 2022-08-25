mod constants;
pub mod infrastructure;
mod library_item;
pub mod node;
#[cfg(feature = "test")]
pub mod test;

use anyhow::{anyhow, Ok, Result};
use chrono::{DateTime, Utc};
use depper::Dependencies;
use infrastructure::{Infrastructure, InfrastructureHelper};
pub use library_item::LibraryItem;
use node::{NodeType, Nodes};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

pub trait Formalize {
    fn formalize(&mut self) -> Result<()>;
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(
        alias = "Scenario",
        alias = "SCENARIO",
        deserialize_with = "deserialize_struct_case_insensitive"
    )]
    pub scenario: Scenario,
}

impl Formalize for Schema {
    fn formalize(&mut self) -> Result<()> {
        self.scenario.formalize()?;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub nodes: Option<Nodes>,
    #[serde(default, rename = "infrastructure", skip_serializing)]
    infrastructure_helper: Option<InfrastructureHelper>,
    #[serde(default, skip_deserializing)]
    pub infrastructure: Option<Infrastructure>,
}

impl Scenario {
    fn map_infrastructure(&mut self) -> Result<()> {
        if let Some(infrastructure_helper) = &self.infrastructure_helper {
            let mut infrastructure = Infrastructure::new();
            for (name, helpernode) in infrastructure_helper.iter() {
                infrastructure.insert(name.to_string(), helpernode.clone().into());
            }
            self.infrastructure = Some(infrastructure);
        }
        Ok(())
    }

    pub fn get_dependencies(&self) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();
        if let Some(nodes_value) = &self.nodes {
            for (node_name, _) in nodes_value.iter() {
                dependency_builder = dependency_builder.add_element(node_name.to_string(), vec![]);
            }
        }
        if let Some(infrastructure) = &self.infrastructure {
            for (node_name, infra_node) in infrastructure.iter() {
                let mut dependencies: Vec<String> = vec![];
                if let Some(links) = &infra_node.links {
                    let links = links
                        .iter()
                        .map(|link| link.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(links.as_slice());
                }
                if let Some(node_dependencies) = &infra_node.dependencies {
                    let node_dependencies = node_dependencies
                        .iter()
                        .map(|dependency| dependency.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(node_dependencies.as_slice());
                }
                dependency_builder =
                    dependency_builder.add_element(node_name.clone(), dependencies);
            }
        }
        dependency_builder.build()
    }

    fn verify_dependencies(&self) -> Result<()> {
        self.get_dependencies()?;
        Ok(())
    }

    fn verify_node_counts(&self) -> Result<()> {
        if let Some(infrastructure) = &self.infrastructure {
            if let Some(nodes) = &self.nodes {
                for (node_name, infra_node) in infrastructure.iter() {
                    if infra_node.count > 1 {
                        if let Some(node) = nodes.get(node_name) {
                            if node.type_field == NodeType::Switch {
                                return Err(anyhow!(
                                    "Node {} is a switch and has count higher than 1",
                                    node_name
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Formalize for Scenario {
    fn formalize(&mut self) -> Result<()> {
        if let Some(mut nodes) = self.nodes.clone() {
            nodes.iter_mut().try_for_each(move |(_, node)| {
                node.formalize()?;
                Ok(())
            })?;
            self.nodes = Some(nodes);
        }
        self.map_infrastructure()?;
        self.verify_node_counts()?;
        self.verify_dependencies()?;
        Ok(())
    }
}

pub fn parse_sdl(sdl_string: &str) -> Result<Schema> {
    let mut schema: Schema = serde_yaml::from_str(sdl_string)?;
    schema.formalize()?;
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_minimal_sdl() {
        let minimal_sdl = r#"
            scenario:
                name: test-scenario
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
        "#;
        let parsed_schema = parse_sdl(minimal_sdl).unwrap();
        insta::assert_yaml_snapshot!(parsed_schema);
    }

    #[test]
    fn includes_a_list_of_nodes() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
        "#;
        let nodes = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }

    #[test]
    fn sdl_keys_are_valid_in_lowercase_uppercase_capitalized() {
        let sdl = r#"
        scenario:
            name: test-scenario
            Description: some-description
            start: 2022-01-20T13:00:00Z
            End: 2022-01-20T23:00:00Z
            nodes:
                Win10:
                    TYPE: VM
                    Description: win-10-description
                    resources:
                        RAM: 4 gib
                        Cpu: 2
                    SOURCE:
                        name: windows10
                        version: '*'

        "#;
        let parsed_schema = parse_sdl(sdl).unwrap();
        insta::assert_yaml_snapshot!(parsed_schema);
    }

    #[test]
    fn includes_infrastructure() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
            infrastructure:
                win10:
                    count: 10
                    dependencies:
                        - deb10
                deb10: 3
        "#;
        let schema = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }
}
