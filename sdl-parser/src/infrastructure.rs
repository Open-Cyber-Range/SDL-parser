use crate::constants::default_node_count;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct InfraNode {
    #[serde(default = "default_node_count", alias = "Count", alias = "COUNT")]
    pub count: u32,
    #[serde(default, alias = "Links", alias = "LINKS")]
    pub links: Option<Vec<String>>,
    #[serde(default, alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
}

impl InfraNode {
    pub fn new(potential_count: Option<u32>) -> Self {
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
    EmptyNode,
    ShortNode(u32),
    LongNode(InfraNode),
}

pub type InfrastructureHelper = HashMap<String, HelperNode>;

pub type Infrastructure = HashMap<String, InfraNode>;

impl From<HelperNode> for InfraNode {
    fn from(helper_node: HelperNode) -> Self {
        match helper_node {
            HelperNode::ShortNode(value) => InfraNode::new(Some(value)),
            HelperNode::LongNode(infranode) => infranode,
            HelperNode::EmptyNode => InfraNode::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_helper(infrastructure_helper: InfrastructureHelper) -> Infrastructure {
        let mut infrastructure: Infrastructure = HashMap::new();
        for (name, helpernode) in infrastructure_helper.iter() {
            infrastructure.insert(name.to_string(), helpernode.clone().into());
        }
        infrastructure
    }

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
            debian-2:
                count: 4      
        "#;
        let infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let infrastructure = from_helper(infrastructure_helper);

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
        let infrastructure = from_helper(infrastructure_helper);

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
        let infrastructure = from_helper(infrastructure_helper);

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
        let infrastructure = from_helper(infrastructure_helper);

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
}
