use crate::{
    constants::{default_node_count, MINIMUM_NODE_COUNT},
    helpers::Connection,
    Formalize,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct InfraNode {
    #[serde(default, alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(default = "default_node_count", alias = "Count", alias = "COUNT")]
    pub count: i32,
    #[serde(default, alias = "Links", alias = "LINKS")]
    pub links: Option<Vec<String>>,
    #[serde(default, alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

impl InfraNode {
    pub fn new(potential_count: Option<i32>) -> Self {
        Self {
            count: match potential_count {
                Some(count) => count,
                None => default_node_count(),
            },
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HelperNode {
    Empty,
    Short(i32),
    Long(InfraNode),
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InfrastructureHelper {
    InfrastructureHelper(HashMap<String, HelperNode>),
}

pub type Infrastructure = HashMap<String, InfraNode>;

impl Formalize for InfraNode {
    fn formalize(&mut self) -> Result<()> {
        if self.count < MINIMUM_NODE_COUNT {
            return Err(anyhow!(
                "Infrastructure Count field can not be less than {MINIMUM_NODE_COUNT}"
            ));
        }
        Ok(())
    }
}

impl From<HelperNode> for InfraNode {
    fn from(helper_node: HelperNode) -> Self {
        match helper_node {
            HelperNode::Short(value) => InfraNode::new(Some(value)),
            HelperNode::Long(infranode) => infranode,
            HelperNode::Empty => InfraNode::new(None),
        }
    }
}

impl From<InfrastructureHelper> for Infrastructure {
    fn from(helper_infrastructure: InfrastructureHelper) -> Self {
        match helper_infrastructure {
            InfrastructureHelper::InfrastructureHelper(helper_infrastructure) => {
                helper_infrastructure
                    .iter()
                    .map(|(node_name, helper_node)| {
                        let infra_node: InfraNode = helper_node.clone().into();
                        (node_name.to_owned(), infra_node)
                    })
                    .collect::<Infrastructure>()
            }
        }
    }
}

impl Connection<Infrastructure> for &String {
    fn validate_connections(&self, potential_node_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(node_names) = potential_node_names {
            if !node_names.contains(self) {
                return Err(anyhow!(
                    "Infrastructure entry {self} does not exist under Nodes"
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn infranode_count_longhand_is_parsed() {
        let sdl = r#"
            count: 23
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(sdl).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_count_shorthand_is_parsed() {
        let sdl = r#"
            23
        "#;
        let infra_node: InfraNode = serde_yaml::from_str::<HelperNode>(sdl).unwrap().into();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_with_links_and_dependencies_is_parsed() {
        let sdl = r#"
            count: 25
            links:
                - switch-2              
            dependencies:
                - windows-10
                - windows-10-vuln-1
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(sdl).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_with_default_count_is_parsed() {
        let sdl = r#"
            links:
                - switch-1
            dependencies:
                - windows-10
                - windows-10-vuln-1 
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(sdl).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn simple_infrastructure_is_parsed() {
        let sdl = r#"
            windows-10-vuln-1:
                count: 10
                description: "A vulnerable Windows 10 machine"
            debian-2:
                count: 4
                description: "A Debian server"     
        "#;
        let infrastructure = serde_yaml::from_str::<Infrastructure>(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(infrastructure);
        });
    }

    #[test]
    fn simple_infrastructure_with_shorthand_is_parsed() {
        let sdl = r#"
            windows-10-vuln-2:
                count: 10
            windows-10-vuln-1: 10
            ubuntu-10: 5
        "#;
        let infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let infrastructure: Infrastructure = infrastructure_helper.into();

        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(infrastructure);
        });
    }

    #[test]
    fn bigger_infrastructure_is_parsed() {
        let sdl = r#"
            switch-1: 1
            windows-10: 3
            windows-10-vuln-1:
                count: 1
            switch-2:
                count: 2
                links:
                    - switch-1
            ubuntu-10:
                links:
                    - switch-1
                dependencies:
                    - windows-10
                    - windows-10-vuln-1
        "#;
        let infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let infrastructure: Infrastructure = infrastructure_helper.into();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(infrastructure);
        });
    }

    #[test]
    fn sdl_keys_are_valid_in_lowercase_uppercase_capitalized() {
        let sdl = r#"
            switch-1: 1
            windows-10: 3
            windows-10-vuln-1:
                Count: 1
                DEPENDENCIES:
                    - windows-10
            switch-2:
                COUNT: 2
                Links:
                    - switch-1
            ubuntu-10:
                LINKS:
                    - switch-1
                Dependencies:
                    - windows-10
                    - windows-10-vuln-1
        "#;
        let infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let infrastructure: Infrastructure = infrastructure_helper.into();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(infrastructure);
        });
    }

    #[test]
    fn empty_count_is_allowed() {
        let sdl = r#"
            switch-1:
        "#;
        serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
    }

    #[should_panic(expected = "Infrastructure Count field can not be less than 1")]
    #[test]
    fn infranode_with_negative_count_is_rejected() {
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
            infrastructure:
                win-10: -1
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[should_panic(expected = "Infrastructure entry debian does not exist under Nodes")]
    #[test]
    fn infranode_with_unknown_name_is_rejected() {
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
            infrastructure:
                debian: 1
        "#;
        parse_sdl(sdl).unwrap();
    }
}
