use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct InfraNode {
    #[serde(default = "default_count", alias = "Count", alias = "COUNT")]
    pub count: u16,
    #[serde(default, alias = "Links", alias = "LINKS")]
    pub links: Option<Vec<String>>,
    #[serde(default, alias = "Dependencies", alias = "DEPENDENCIES")]
    pub dependencies: Option<Vec<String>>,
}

fn default_count() -> u16 {
    1
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Schema {
    #[serde(alias = "Infrastructure", alias = "INFRASTRUCTURE", default)]
    pub infrastructure: HashMap<String, InfraNode>,
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
    fn bigger_infrastructure_is_parsed() {
        let infrastructure = r#"
            infrastructure:
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
        let schema = serde_yaml::from_str::<Schema>(infrastructure).unwrap();
        insta::with_settings!({sort_maps => true}, {
            insta::assert_yaml_snapshot!(schema);
        });
    }
}
