use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::node::NodeMap;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Infrastructure {
  pub description: Option<String>,
  pub node: Option<NodeMap>
}

pub type InfrastructureMap = HashMap<String, Infrastructure>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn includes_minimal_node_requirements() {
    let node_sdl = r#"
        test-infrastructure:
          description: my-test-description
        "#;
    let parsed_infrastructure = serde_yaml::from_str::<InfrastructureMap>(node_sdl).unwrap();

    let name = (*parsed_infrastructure.keys().collect::<Vec<&String>>().get(0).unwrap()).clone();
    let node = (*parsed_infrastructure.values().collect::<Vec<&Infrastructure>>().get(0).unwrap()).clone();
    
    assert_eq!(name, "test-infrastructure".to_string());
    assert_eq!(node.description, Some("my-test-description".to_string()));
  }
}
