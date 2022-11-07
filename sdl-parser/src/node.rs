use crate::{helpers::Connection, vulnerability::Vulnerability, Formalize};
use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

use crate::common::{HelperSource, Source};

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
    Switch,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Resources {
    #[serde(deserialize_with = "parse_bytesize")]
    pub ram: u64,
    pub cpu: u32,
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
    source_helper: Option<HelperSource>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
    #[serde(default, alias = "Features", alias = "FEATURES")]
    pub features: Option<HashMap<String, String>>,
    #[serde(default, alias = "Conditions", alias = "CONDITIONS")]
    pub conditions: Option<Vec<String>>,
    #[serde(default, alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Option<Vec<String>>,
    pub roles: Option<HashMap<String, String>>,
}

impl Connection<Vulnerability> for (&String, &Node) {
    fn validate_connections(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()> {
        if let Some(node_vulnerabilities) = &self.1.vulnerabilities {
            if let Some(vulnerabilities) = potential_vulnerability_names {
                for node_vulnerability in node_vulnerabilities.iter() {
                    if !vulnerabilities.contains(node_vulnerability) {
                        return Err(anyhow!(
                            "Vulnerability {} not found under scenario",
                            node_vulnerability
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Vulnerability list is empty under scenario, but node {} has vulnerabilities",
                    self.0
                ));
            }
        }
        Ok(())
    }
}

impl Formalize for Node {
    fn formalize(&mut self) -> Result<()> {
        if let Some(source_helper) = &self.source_helper {
            self.source = Some(source_helper.to_owned().into());
            return Ok(());
        } else if self.type_field == NodeType::VM {
            return Err(anyhow::anyhow!("No source found"));
        }
        Ok(())
    }
}

pub type Nodes = HashMap<String, Node>;

#[cfg(test)]
mod tests {
    use crate::parse_sdl;

    use super::*;

    #[test]
    fn vm_source_fields_are_mapped_correctly() {
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
                        version: 1.2.3

        "#;
        let nodes = parse_sdl(sdl).unwrap().scenario.nodes;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }

    #[test]
    fn vm_source_longhand_is_parsed() {
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
    fn vm_source_shorthand_is_parsed() {
        let shorthand_source = r#"
            type: VM
            source: package-name

        "#;
        let node = serde_yaml::from_str::<Node>(shorthand_source).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn node_conditions_are_parsed() {
        let node_sdl = r#"
            type: VM
            roles:
                admin: "username"
                moderator: "name"     
            conditions:
                condition-1: "admin"
                condition-2: "moderator"

        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn switch_source_is_not_required() {
        let shorthand_source = r#"
            type: Switch

        "#;
        serde_yaml::from_str::<Node>(shorthand_source)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    fn includes_node_requirements_with_switch_type() {
        let node_sdl = r#"
            type: Switch
            description: a network switch

        "#;
        let node = serde_yaml::from_str::<Node>(node_sdl).unwrap();
        insta::assert_debug_snapshot!(node);
    }

    #[test]
    fn includes_nodes_with_defined_features() {
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
                    roles:
                        admin: "username"
                        moderator: "name"
                    features:
                        feature-1: "admin" 
                        feature-2: "moderator" 
            features:
                feature-1:
                    type: service
                    source: dl-library
                feature-2:
                    type: artifact
                    source:
                        name: my-cool-artifact
                        version: 1.0.0
                    
        "#;
        let scenario = parse_sdl(sdl).unwrap().scenario;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }

    #[test]
    #[should_panic]
    fn roles_missing_when_features_exist() {
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
                    features:
                        feature-1: "admin"
            features:
                feature-1:
                    type: service
                    source: dl-library

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn role_under_feature_missing_from_node() {
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
                    roles:
                        moderator: "name"
                    features:
                        feature-1: "admin"
            features:
                feature-1:
                    type: service
                    source: dl-library

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn same_name_for_role_only_saves_one_role() {
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
                    roles:
                        admin: "username"
                        admin: "username2"

        "#;
        let scenario = parse_sdl(sdl).unwrap().scenario;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }
}
