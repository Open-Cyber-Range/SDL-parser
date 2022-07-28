use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_count() -> u32 {
    1
}

fn new_infranode(count_value: u32) -> InfraNode {
    InfraNode {
        count: count_value,
        ..Default::default()
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct InfraNode {
    #[serde(default = "default_count", alias = "Count", alias = "COUNT")]
    pub count: u32,
    #[serde(default, alias = "Links", alias = "LINKS")]
    pub links: Option<Vec<String>>,
    #[serde(default, alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HelperNode {
    ShortNode(u32),
    LongNode(InfraNode),
}

impl HelperNode {
    pub fn map_into_infranode(&mut self) -> InfraNode {
        match self {
            HelperNode::ShortNode(value) => new_infranode(*value),
            HelperNode::LongNode(infranode) => infranode.clone(),
        }
    }
}

pub type InfrastructureHelper = HashMap<String, HelperNode>;

pub type Infrastructure = HashMap<String, InfraNode>;

#[cfg(test)]
mod tests {
    use super::*;

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
        let infra_node = serde_yaml::from_str::<HelperNode>(sdl)
            .unwrap()
            .map_into_infranode();
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
        let mut infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let mut infrastructure: Infrastructure = HashMap::new();
        for (name, helpernode) in infrastructure_helper.iter_mut() {
            infrastructure.insert(name.to_string(), helpernode.map_into_infranode());
        }
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
        let mut infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let mut infrastructure: Infrastructure = HashMap::new();
        for (name, helpernode) in infrastructure_helper.iter_mut() {
            infrastructure.insert(name.to_string(), helpernode.map_into_infranode());
        }
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
        let mut infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let mut infrastructure: Infrastructure = HashMap::new();
        for (name, helpernode) in infrastructure_helper.iter_mut() {
            infrastructure.insert(name.to_string(), helpernode.map_into_infranode());
        }
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
        let mut infrastructure_helper = serde_yaml::from_str::<InfrastructureHelper>(sdl).unwrap();
        let mut infrastructure: Infrastructure = HashMap::new();
        for (name, helpernode) in infrastructure_helper.iter_mut() {
            infrastructure.insert(name.to_string(), helpernode.map_into_infranode());
        }
        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(infrastructure);
        });
    }
}
