pub mod infrastructure;
mod library_item;
pub mod node;
#[cfg(feature = "test")]
pub mod test;

use anyhow::Result;
use chrono::{DateTime, Utc};
use infrastructure::{Infrastructure, InfrastructureHelper};
pub use library_item::LibraryItem;
use node::NodeMap;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub nodes: Option<NodeMap>,
    #[serde(default, rename = "infrastructure", skip_serializing)]
    _infrastructure_helper: Option<InfrastructureHelper>,
    #[serde(default, skip_deserializing)]
    pub infrastructure: Option<Infrastructure>,
}

impl Scenario {
    pub fn map_infrastructure(
        &mut self,
        mut infrastructure_helper: InfrastructureHelper,
    ) -> Infrastructure {
        let mut infrastructure = Infrastructure::new();
        for (name, helpernode) in infrastructure_helper.iter_mut() {
            infrastructure.insert(name.to_string(), helpernode.map_into_infranode());
        }
        infrastructure
    }
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

pub fn parse_sdl(sdl_string: &str) -> Result<Schema> {
    let mut schema: Schema = serde_yaml::from_str(sdl_string)?;
    if let Some(nodes) = &mut schema.scenario.nodes {
        nodes.iter_mut().for_each(|(_, node)| node.map_source());
    }
    if let Some(infrastructure_helper) = &schema.scenario._infrastructure_helper {
        schema.scenario.infrastructure = Some(
            schema
                .scenario
                .clone()
                .map_infrastructure(infrastructure_helper.clone()),
        );
    }
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
        let nodes = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }
}
