pub mod capability;
pub mod common;
pub mod condition;
mod constants;
pub mod feature;
mod helpers;
pub mod infrastructure;
mod library_item;
pub mod node;
#[cfg(feature = "test")]
pub mod test;
pub mod vulnerability;

use crate::helpers::Connection;
use anyhow::{anyhow, Ok, Result};
use capability::Capabilities;
use chrono::{DateTime, Utc};
use condition::{Condition, Conditions};
use constants::MAX_LONG_NAME;
use depper::Dependencies;
use feature::{Feature, Features};
use infrastructure::{Infrastructure, InfrastructureHelper};
pub use library_item::LibraryItem;
use node::{NodeType, Nodes};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use vulnerability::{Vulnerabilities, Vulnerability};

pub trait Formalize {
    fn formalize(&mut self) -> Result<()>;
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Schema {
    #[serde(
        alias = "Scenario",
        alias = "SCENARIO",
        deserialize_with = "deserialize_struct_case_insensitive"
    )]
    pub scenario: Scenario,
}

impl Schema {
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(&self).map_err(|e| anyhow!("Failed to serialize to yaml: {}", e))
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let mut schema: Self = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow!("Failed to deserialize from yaml: {}", e))?;
        schema.formalize()?;
        Ok(schema)
    }
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
    pub features: Option<Features>,
    #[serde(default, rename = "infrastructure", skip_serializing)]
    infrastructure_helper: Option<InfrastructureHelper>,
    #[serde(default, skip_deserializing)]
    pub infrastructure: Option<Infrastructure>,
    pub conditions: Option<Conditions>,
    pub vulnerabilities: Option<Vulnerabilities>,
    pub capabilities: Option<Capabilities>,
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

    pub fn get_node_dependencies(&self) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();
        if let Some(nodes_value) = &self.nodes {
            for (node_name, _) in nodes_value.iter() {
                dependency_builder = dependency_builder.add_element(node_name.to_string(), vec![]);
            }
        }

        self.build_infrastructure_dependencies(dependency_builder)
    }

    pub fn get_feature_dependencies(&self) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();
        if let Some(features_value) = &self.features {
            for (feature_name, _) in features_value.iter() {
                dependency_builder =
                    dependency_builder.add_element(feature_name.to_string(), vec![]);
            }
        }
        self.build_feature_dependencies(dependency_builder)
    }

    fn build_infrastructure_dependencies(
        &self,
        mut dependency_builder: depper::DependenciesBuilder,
    ) -> Result<Dependencies, anyhow::Error> {
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

    fn build_feature_dependencies(
        &self,
        mut dependency_builder: depper::DependenciesBuilder,
    ) -> Result<Dependencies, anyhow::Error> {
        if let Some(features) = &self.features {
            for (feature_name, feature) in features.iter() {
                let mut dependencies: Vec<String> = vec![];
                if let Some(links) = &feature.dependencies {
                    let links = links
                        .iter()
                        .map(|dependency| dependency.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(links.as_slice());
                }
                if let Some(feature_dependencies) = &feature.dependencies {
                    let feature_dependencies = feature_dependencies
                        .iter()
                        .map(|dependency| dependency.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(feature_dependencies.as_slice());
                }
                dependency_builder =
                    dependency_builder.add_element(feature_name.clone(), dependencies);
            }
        }
        dependency_builder.build()
    }

    fn verify_dependencies(&self) -> Result<()> {
        self.get_node_dependencies()?;
        self.get_feature_dependencies()?;
        Ok(())
    }

    fn verify_switch_counts(&self) -> Result<()> {
        if let Some(infrastructure) = &self.infrastructure {
            if let Some(nodes) = &self.nodes {
                for (node_name, infra_node) in infrastructure.iter() {
                    if infra_node.count > 1 {
                        if let Some(node) = nodes.get(node_name) {
                            if node.type_field == NodeType::Switch {
                                return Err(anyhow!(
                                    "Node {} is a switch with a count higher than 1",
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

    pub fn verify_node_name_length(&self) -> Result<()> {
        if let Some(nodes) = &self.nodes {
            for name in nodes.keys() {
                if name.len() > MAX_LONG_NAME {
                    return Err(anyhow!(
                        "{} is too long, maximum node name length is {:?}",
                        name,
                        MAX_LONG_NAME
                    ));
                }
            }
        }
        Ok(())
    }

    fn verify_features(&self) -> Result<()> {
        let feature_names = self
            .features
            .as_ref()
            .map(|feature_map| feature_map.keys().cloned().collect::<Vec<String>>());
        if let Some(nodes) = &self.nodes {
            for combined_value in nodes.iter() {
                Connection::<Feature>::validate_connections(&combined_value, &feature_names)?;
            }
        }
        Ok(())
    }

    fn verify_conditions(&self) -> Result<()> {
        let condition_names = self
            .conditions
            .as_ref()
            .map(|condition_map| condition_map.keys().cloned().collect::<Vec<String>>());
        if let Some(nodes) = &self.nodes {
            for (node_name, node) in nodes.iter() {
                if let Some(node_conditions) = &node.conditions {
                    if self.conditions.is_none() {
                        return Err(anyhow!(
                            "Conditions list is empty under scenario, but node {} has conditions",
                            node_name
                        ));
                    }
                    if let Some(conditions) = &self.conditions {
                        for node_condition in node_conditions.iter() {
                            if !conditions.contains_key(node_condition) {
                                return Err(anyhow!(
                                    "Condition {} not found under scenario",
                                    node_condition
                                ));
                            }
                        }
                    }
                    if let Some(infrastructure) = &self.infrastructure {
                        if let Some(infra_node) = infrastructure.get(node_name) {
                            if infra_node.count != 1 {
                                return Err(anyhow!(
                                    "Condition VM {} has a count other than 1",
                                    node_name
                                ));
                            }
                        }
                    }
                }
            }
        }
        if let Some(capabilities) = &self.capabilities {
            for combined_value in capabilities.iter() {
                Connection::<Condition>::validate_connections(&combined_value, &condition_names)?;
            }
        }
        Ok(())
    }

    fn verify_vulnerabilities(&self) -> Result<()> {
        let vulnernability_names = self
            .vulnerabilities
            .as_ref()
            .map(|vulnerability_map| vulnerability_map.keys().cloned().collect::<Vec<String>>());
        if let Some(nodes) = &self.nodes {
            for combined_value in nodes.iter() {
                Connection::<Vulnerability>::validate_connections(
                    &combined_value,
                    &vulnernability_names,
                )?;
            }
        }
        if let Some(features) = &self.features {
            for combined_value in features.iter() {
                combined_value.validate_connections(&vulnernability_names)?;
            }
        }
        if let Some(capabilities) = &self.capabilities {
            for combined_value in capabilities.iter() {
                Connection::<Vulnerability>::validate_connections(
                    &combined_value,
                    &vulnernability_names,
                )?;
            }
        }
        Ok(())
    }

    fn verify_roles(&self) -> Result<()> {
        if let Some(nodes) = &self.nodes {
            for (node_name, node) in nodes.iter() {
                if let Some(features) = &node.features {
                    if let Some(roles) = &node.roles {
                        for feature_role in features.values() {
                            if !roles.contains_key(feature_role) {
                                return Err(anyhow!(
                                    "Role {} not found in node {}",
                                    feature_role,
                                    node_name
                                ));
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!("No roles found for feature(s)"));
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

        if let Some(features) = &mut self.features {
            features.iter_mut().try_for_each(move |(_, feature)| {
                feature.formalize()?;
                Ok(())
            })?;
        }

        if let Some(mut conditions) = self.conditions.clone() {
            conditions.iter_mut().try_for_each(move |(_, condition)| {
                condition.formalize()?;
                Ok(())
            })?;
            self.conditions = Some(conditions);
        }

        if let Some(mut capabilities) = self.capabilities.clone() {
            capabilities
                .iter_mut()
                .try_for_each(move |(_, condition)| {
                    condition.formalize()?;
                    Ok(())
                })?;
        }
        self.map_infrastructure()?;
        self.verify_node_name_length()?;
        self.verify_switch_counts()?;
        self.verify_features()?;
        self.verify_conditions()?;
        self.verify_dependencies()?;
        self.verify_vulnerabilities()?;
        self.verify_roles()?;
        Ok(())
    }
}

pub fn parse_sdl(sdl_string: &str) -> Result<Schema> {
    Schema::from_yaml(sdl_string)
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
    fn includes_nodes() {
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
        let nodes = parse_sdl(sdl).unwrap().scenario.nodes;
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
        parse_sdl(sdl).unwrap();
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
        let infrastrcture = parse_sdl(sdl).unwrap().scenario.infrastructure;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(infrastrcture);
        });
    }

    #[test]
    fn includes_features() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-cool-service:
                    type: service
                    source: some-service
                my-cool-config:
                    type: configuration
                    source: some-configuration
                    dependencies:
                        - my-cool-service
                my-cool-artifact:
                    type: artifact
                    source:
                        name: dl_library
                        version: 1.2.3
                    dependencies: 
                        - my-cool-service
        "#;
        let features = parse_sdl(sdl).unwrap().scenario.features;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(features);
        });
    }

    #[test]
    fn includes_conditions() {
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
                    conditions:
                        - condition-1
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
                    conditions:
                        - condition-2
                        - condition-3
            infrastructure:
                win10:
                    count: 1
                    dependencies:
                        - deb10
                deb10: 1
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                condition-2:
                    source: digital-library-package
                condition-3:
                    command: executable/path.sh
                    interval: 30

        "#;
        let conditions = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }

    #[test]
    #[should_panic]
    fn condition_vm_count_in_infrastructure_over_1() {
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
                    conditions:
                        - condition-1
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
                    count: 3
                    dependencies:
                        - deb10
                deb10: 1
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn condition_doesnt_exist() {
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
                    conditions:
                        - condition-1
            infrastructure:
                win10: 1

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn too_long_node_name_is_disallowed() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                my-really-really-superlong-non-compliant-name:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                    conditions:
                        - condition-1
        "#;
        parse_sdl(sdl).unwrap();
    }
}
