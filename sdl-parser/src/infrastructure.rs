use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct InfraNode {
    #[serde(default = "default_count", alias = "Count", alias = "COUNT")]
    pub count: u16,
}

fn default_count() -> u16 {
    1
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
}
