use crate::node::Node;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct LibraryItem {
    name: String,
    version: String,
}

pub fn generate_package_list(sdl_string: &str) -> Result<Vec<LibraryItem>> {
    let nodes: Vec<Node> = serde_yaml::from_str(sdl_string)?;
    let mut result = Vec::new();
    for val in nodes {
        if val.source.clone().is_some() && val.source.clone().unwrap().package.is_some() {
            let item = LibraryItem {
                name: val.source.clone().unwrap().package.unwrap().name,
                version: val.source.unwrap().package.unwrap().version,
            };
            result.push(item);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_package_list_based_on_sdl() {
        let node_sdl = r#"
              - type: VM
                template: windows10
                description: win10 node for OCR
                flavor:
                    ram: 4gb
                    cpu: 4
                source:
                    package:
                        name: basic-windows10
                        version: '*'
              - type: VM
                template: debian10
                description: deb-10-description
                flavor:
                    ram: 4gb
                    cpu: 4
                source:
                    package:
                        name: debian10
                        version: '*'
              - type: VM
                template: windows10
                description: win10 node for OCR
                flavor:
                    ram: 4gb
                    cpu: 4
                source:
                    template: windows10-template
              - type: VM
                template: windows10
                description: win10 node for OCR
                flavor:
                    ram: 4gb
                    cpu: 4
        "#;
        
        let library_items = generate_package_list(node_sdl).unwrap();
        let expected_answer = vec![
            LibraryItem {
                name: "basic-windows10".to_string(),
                version: "*".to_string(),
            },
            LibraryItem {
                name: "debian10".to_string(),
                version: "*".to_string(),
            },
        ];
        assert_eq!(library_items, expected_answer)
    }
}
