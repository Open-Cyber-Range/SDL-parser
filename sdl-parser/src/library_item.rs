use crate::node::Node;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct LibraryItem {
    pub name: String,
    pub version: String,
}

impl LibraryItem {
    fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

pub fn generate_package_list(sdl_string: &str) -> Result<Vec<LibraryItem>> {
    let nodes: Vec<Node> = serde_yaml::from_str(sdl_string)?;
    let mut result = Vec::new();

    for node in nodes {
        if let Some(source) = node.source {
            if let Some(package) = source.package {
                result.push(LibraryItem::new(package.name, package.version));
            }
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
        insta::assert_debug_snapshot!(library_items);
    }
}
