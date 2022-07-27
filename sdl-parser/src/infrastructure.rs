use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_count() -> u32 {
    1
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct LongNode {
    #[serde(default = "default_count", alias = "Count", alias = "COUNT")]
    pub count: u32,
    #[serde(default, alias = "Links", alias = "LINKS")]
    pub links: Option<Vec<String>>,
    #[serde(default, alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InfraNode {
    ShortNode(u32),
    LongNode(LongNode),
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct RawSchema {
    #[serde(alias = "Infrastructure", alias = "INFRASTRUCTURE", default)]
    pub infrastructure: HashMap<String, InfraNode>,
}

impl RawSchema {
    pub fn remap_nodes(&self) -> Schema {
        let mut schema = Schema {
            ..Default::default()
        };
        for node in &self.infrastructure {
            match node.1 {
                InfraNode::ShortNode(value) => {
                    schema.infrastructure.insert(
                        node.0.to_string(),
                        LongNode {
                            count: *value,
                            ..Default::default()
                        },
                    );
                }
                InfraNode::LongNode(long_node) => {
                    schema
                        .infrastructure
                        .insert(node.0.to_string(), long_node.to_owned());
                }
            }
        }
        schema
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Schema {
    #[serde(alias = "Infrastructure", alias = "INFRASTRUCTURE", default)]
    pub infrastructure: HashMap<String, LongNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infranode_count_longhand_is_parsed() {
        let longhand_count = r#"
            count: 23
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(longhand_count).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_count_shorthand_is_parsed() {
        let shorthand_count = r#"
            23
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(shorthand_count).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_with_links_and_dependencies_is_parsed() {
        let infranode_with_links_and_dependencies = r#"
            count: 25
            links:
                - switch-2              
            dependencies:
                - windows-10
                - windows-10-vuln-1
        "#;
        let infra_node =
            serde_yaml::from_str::<InfraNode>(infranode_with_links_and_dependencies).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn infranode_with_default_count_is_parsed() {
        let infranode_with_default_count = r#"
            links:
                - switch-1
            dependencies:
                - windows-10
                - windows-10-vuln-1 
        "#;
        let infra_node = serde_yaml::from_str::<InfraNode>(infranode_with_default_count).unwrap();
        insta::assert_debug_snapshot!(infra_node);
    }

    #[test]
    fn simple_infrastructure_is_parsed() {
        let simple_infrastructure = r#"
            infrastructure:
                windows-10-vuln-1:
                    count: 10
                debian-2:
                    count: 4      
        "#;
        let schema = serde_yaml::from_str::<Schema>(simple_infrastructure).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    fn simple_infrastructure_with_shorthand_is_parsed() {
        let simple_infrastructure_with_shorthand = r#"
            infrastructure:
                windows-10-vuln-2:
                    count: 10
                windows-10-vuln-1: 10
                ubuntu-10: 5   
        "#;
        let raw_schema =
            serde_yaml::from_str::<RawSchema>(simple_infrastructure_with_shorthand).unwrap();
        let schema = raw_schema.remap_nodes();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    fn bigger_infrastructure_is_parsed() {
        let bigger_infrastructure = r#"
            infrastructure:
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
        let raw_schema = serde_yaml::from_str::<RawSchema>(bigger_infrastructure).unwrap();
        let schema = raw_schema.remap_nodes();
        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    fn sdl_keys_are_valid_in_lowercase_uppercase_capitalized() {
        let infrastructure = r#"
            INFRASTRUCTURE:
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
        let raw_schema = serde_yaml::from_str::<RawSchema>(infrastructure).unwrap();
        let schema = raw_schema.remap_nodes();
        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(schema);
        });
    }
}
