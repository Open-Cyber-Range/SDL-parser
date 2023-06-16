use crate::{
    condition::Condition, entity::Entity, feature::Feature, helpers::Connection,
    infrastructure::Infrastructure, vulnerability::Vulnerability, Formalize,
};
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
        .0)
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
pub struct Role {
    #[serde(alias = "Username", alias = "USERNAME")]
    pub username: String,
    #[serde(alias = "Entity", alias = "ENTITY")]
    pub entities: Option<Vec<String>>,
}

pub type Roles = HashMap<String, Role>;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub type_field: NodeType,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
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
    _source_helper: Option<HelperSource>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
    #[serde(default, alias = "Features", alias = "FEATURES")]
    pub features: Option<HashMap<String, String>>,
    #[serde(default, alias = "Conditions", alias = "CONDITIONS")]
    pub conditions: Option<HashMap<String, String>>,
    #[serde(default, alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Option<Vec<String>>,
    #[serde(
        default,
        rename = "roles",
        alias = "Roles",
        alias = "ROLES",
        skip_serializing
    )]
    _roles_helper: Option<HelperRoles>,
    #[serde(skip_deserializing)]
    pub roles: Option<Roles>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RoleTypes {
    Username(String),
    Role(Role),
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HelperRoles {
    MixedRoles(HashMap<String, RoleTypes>),
}

impl From<HelperRoles> for Roles {
    fn from(helper_role: HelperRoles) -> Self {
        match helper_role {
            HelperRoles::MixedRoles(mixed_role) => mixed_role
                .into_iter()
                .map(|(role_name, role_value)| {
                    let role_value = match role_value {
                        RoleTypes::Role(role) => role,
                        RoleTypes::Username(username) => Role {
                            username,
                            entities: None,
                        },
                    };
                    (role_name, role_value)
                })
                .collect::<Roles>(),
        }
    }
}

impl Connection<Vulnerability> for (&String, &Node) {
    fn validate_connections(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()> {
        if let Some(node_vulnerabilities) = &self.1.vulnerabilities {
            if let Some(vulnerabilities) = potential_vulnerability_names {
                for vulnerability_name in node_vulnerabilities.iter() {
                    if !vulnerabilities.contains(vulnerability_name) {
                        return Err(anyhow!(
                            "Vulnerability \"{vulnerability_name}\" not found under Scenario Vulnerabilities",
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Node \"{node_name}\" has Vulnerabilities but none found under Scenario",
                    node_name = self.0
                ));
            }
        }
        Ok(())
    }
}

impl Connection<Feature> for (&String, &Node) {
    fn validate_connections(&self, potential_feature_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(node_features) = &self.1.features {
            if let Some(features) = potential_feature_names {
                for node_feature in node_features.keys() {
                    if !features.contains(node_feature) {
                        return Err(anyhow!(
                            "Node \"{node_name}\" Feature \"{node_feature}\" not found under Scenario Features",
                            node_name = &self.0,
                        ));
                    }
                }
            } else if !node_features.is_empty() {
                return Err(anyhow!(
                    "Node \"{node_name}\" has Features but none found under Scenario",
                    node_name = &self.0,
                ));
            }
        }
        Ok(())
    }
}

impl Connection<Condition> for (&String, &Node, &Option<Infrastructure>) {
    fn validate_connections(&self, potential_condition_names: &Option<Vec<String>>) -> Result<()> {
        let (node_name, node, infrastructure) = self;
        if let Some(node_conditions) = &node.conditions {
            if let Some(conditions) = potential_condition_names {
                for condition_name in node_conditions.keys() {
                    if !conditions.contains(condition_name) {
                        return Err(anyhow!(
                            "Node \"{node_name}\" Condition \"{condition_name}\" not found under Scenario Conditions"
                        ));
                    }
                }
                if node_conditions.keys().len() > 0 {
                    if let Some(infrastructure) = infrastructure {
                        if let Some(infra_node) = infrastructure.get(node_name.to_owned()) {
                            if infra_node.count > 1 {
                                return Err(anyhow!(
                                    "Node \"{node_name}\" can not have count bigger than 1, if it has conditions defined"
                                ));
                            }
                        }
                    }
                }
            } else if !node_conditions.is_empty() {
                return Err(anyhow!(
                    "Node \"{node_name}\" has Conditions but none found under Scenario"
                ));
            }
        }
        Ok(())
    }
}

impl Connection<Node> for (&String, &Option<Roles>) {
    fn validate_connections(&self, potential_role_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(role_names) = potential_role_names {
            if let Some(roles) = self.1 {
                for role_name in role_names {
                    if !roles.contains_key(role_name) {
                        return Err(anyhow!(
                            "Role {role_name} not found under for Node {node_name}'s roles",
                            node_name = self.0
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Roles list is empty for Node {node_name} but it has Role requirements",
                    node_name = self.0
                ));
            }
        }

        Ok(())
    }
}

impl Connection<Entity> for (&String, &Option<HashMap<String, Role>>) {
    fn validate_connections(&self, potential_entity_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(node_roles) = self.1 {
            for role in node_roles.values() {
                if let Some(role_entities) = &role.entities {
                    if let Some(entity_names) = potential_entity_names {
                        for role_entity in role_entities {
                            if !entity_names.contains(role_entity) {
                                return Err(anyhow!(
                                "Role Entity {role_entity} for Node {node_name} not found under Entities",
                                node_name = self.0
                            ));
                            }
                        }
                    } else {
                        return Err(anyhow!(
                            "Entities list under Scenario is empty but Node {node_name} has Role Entities",
                            node_name = self.0
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Formalize for Node {
    fn formalize(&mut self) -> Result<()> {
        if self.type_field == NodeType::VM {
            if let Some(source_helper) = &self._source_helper {
                self.source = Some(source_helper.to_owned().into());
            } else {
                return Err(anyhow::anyhow!("A Node is missing a source field"));
            }
            if self.resources.is_none() {
                return Err(anyhow::anyhow!(
                    "Nodes of type VM must have Resources defined"
                ));
            }
        } else if self.type_field == NodeType::Switch {
            if self._source_helper.is_some() {
                return Err(anyhow::anyhow!("Nodes of type Switch can not have Sources"));
            }

            if self.resources.is_some() {
                return Err(anyhow::anyhow!(
                    "Nodes of type Switch can not have Resources"
                ));
            }
        }
        if let Some(helper_roles) = &self._roles_helper {
            self.roles = Some(helper_roles.to_owned().into());
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
                deb-10:
                    type: VM
                    resources:
                        ram: 2 gib
                        cpu: 2
                    source:
                        name: debian10
                        version: 1.2.3

        "#;
        let nodes = parse_sdl(sdl).unwrap().nodes;
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
            conditions:
                condition-1: "admin"

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
        let scenario = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }

    #[test]
    #[should_panic(expected = "Roles list is empty for Node win-10 but it has Role requirements")]
    fn roles_missing_when_features_exist() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                    resources:
                        ram: 4 GiB
                        cpu: 2
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
    #[should_panic(expected = "Role admin not found under for Node win-10's roles")]
    fn role_under_feature_missing_from_node() {
        let sdl = r#"
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
                        admin: "username2"

        "#;
        let scenario = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }

    #[test]
    fn nested_node_role_entity_found_under_entities() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources:
                        cpu: 2
                        ram: 32 gib
                    source: windows10
                    roles:
                        admin: 
                            username: "admin"
                            entities:
                                - blue-team.bob
            entities:
                blue-team:
                    name: The Blue Team
                    entities:
                        bob:
                            name: Blue Bob
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Role Entity blue-team.bob for Node win-10 not found under Entities")]
    fn entity_missing_for_node_role_entity() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources:
                        cpu: 2
                        ram: 32 gib
                    source: windows10
                    roles:
                        admin: 
                            username: "admin"
                            entities:
                                - blue-team.bob
            entities:
                blue-team:
                    name: The Blue Team
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Entities list under Scenario is empty but Node win-10 has Role Entities")]
    fn entities_missing_while_node_has_role_entity() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources:
                        cpu: 2
                        ram: 32 gib
                    source: windows10
                    roles:
                        admin: 
                            username: "admin"
                            entities:
                                - blue-team.bob
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn can_parse_shorthand_node_roles() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources:
                        cpu: 2
                        ram: 32 gib
                    source: windows10
                    roles:
                        admin: admin
        "#;
        let scenario = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }
    #[test]
    fn can_parse_longhand_node_roles() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    resources: 
                        cpu: 2
                        ram: 2 gib
                    source: windows10
                    roles:
                        user: 
                            username: user
        "#;
        let scenario = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(scenario);
        });
    }
    #[test]
    fn can_parse_mixed_short_and_longhand_node_roles() {
        let sdl = r#"
name: test-scenario
description: some-description
start: 2022-01-20T13:00:00Z
end: 2022-01-20T23:00:00Z
nodes:
    win-10:
        type: VM
        resources: 
            cpu: 2
            ram: 2 gib
        source: windows10
        roles:
            admin: admin
            user: 
                username: user
                entities:
                    - blue-team.bob

entities:
    blue-team:
        name: The Blue Team
        entities:
            bob:
                name: Blue Bob
        
        "#;
        let parsed_sdl = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(parsed_sdl);
        });
    }

    #[test]
    #[should_panic(expected = "Nodes of type VM must have Resources defined")]
    fn resources_missing_for_vm_node() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Nodes of type Switch can not have Sources")]
    fn source_defined_for_switch_node() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                switch-1:
                    type: Switch
                    source: windows10

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Nodes of type Switch can not have Resources")]
    fn resources_defined_for_switch_node() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                switch-1:
                    type: Switch
                    resources: 
                        cpu: 2
                        ram: 2 gib

        "#;
        parse_sdl(sdl).unwrap();
    }
}
