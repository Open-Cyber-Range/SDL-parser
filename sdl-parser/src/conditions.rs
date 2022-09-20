use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    common::{HelperSource, Source},
    infrastructure::Infrastructure,
    Formalize,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(rename = "vm-name", alias = "Vm-name", alias = "VM-NAME")]
    pub vm_name: String,
    #[serde(default, alias = "Command", alias = "COMMAND")]
    pub command: Option<String>,
    #[serde(default, alias = "Interval", alias = "INTERVAL")]
    pub interval: Option<u32>,
    #[serde(
        default,
        rename = "source",
        alias = "Source",
        alias = "SOURCE",
        skip_serializing
    )]
    source_helper: Option<HelperSource>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
}

impl Formalize for Condition {
    fn formalize(&mut self) -> Result<()> {
        if self.command.is_some() && self.interval.is_some() {
            self.source_helper = None;
            return Ok(());
        } else if let Some(source_helper) = &self.source_helper {
            self.source = Some(source_helper.to_owned().into());
            return Ok(());
        }
        Ok(())
    }
}

impl Condition {
    pub fn check_vm_count(&self, infrastructure: &Infrastructure) -> Result<()> {
        for (node_name, infra_node) in infrastructure.iter() {
            if &self.vm_name == node_name {
                if infra_node.count != 1 {
                    return Err(anyhow!(
                        "Condition VM {} has a count other than 1",
                        node_name
                    ));
                } else {
                    return Ok(());
                }
            }
        }
        Err(anyhow!(
            "Condition VM {} not found under infrastructure",
            &self.vm_name
        ))
    }
}

pub type Conditions = HashMap<String, Condition>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn source_fields_are_mapped_correctly() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                deb-10:
                    type: VM
                    source:
                        name: debian10
                        version: '*'
            conditions:
                condition-1:
                    vm-name: win-10
                    command: executable/path.sh
                    interval: 30
                condition-2:
                    vm-name: deb-10
                    source: digital-library-package

        "#;
        let schema = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    fn command_condition_is_parsed() {
        let sdl = r#"
            vm-name: windows-10
            command: executable/path.sh
            interval: 30       

        "#;
        let condition = serde_yaml::from_str::<Condition>(sdl).unwrap();
        insta::assert_debug_snapshot!(condition);
    }

    #[test]
    fn library_condition_is_parsed() {
        let sdl = r#"
            vm-name: green-evaluation-machine
            source: digital-library-package 

        "#;
        let condition = serde_yaml::from_str::<Condition>(sdl).unwrap();
        insta::assert_debug_snapshot!(condition);
    }

    #[test]
    fn command_condition_is_parsed_correctly_even_with_source() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                deb-10:
                    type: VM
                    source:
                        name: debian10
                        version: '*'
            conditions:
                condition-1:
                    vm-name: win-10
                    command: executable/path.sh
                    interval: 30
                    source: digital-library-package
                condition-2:
                    vm-name: deb-10
                    source: digital-library-package

        "#;
        let schema = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    #[should_panic]
    fn condition_vm_count_in_infrastructure_over_1() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
            infrastructure:
                win10:
                    count: 3
                    dependencies:
                        - deb10
                deb10: 1
            conditions:
                condition-1:
                    vm-name: win10
                    command: executable/path.sh
                    interval: 30
                condition-2:
                    vm-name: deb10
                    source: digital-library-package
                condition-3:
                    vm-name: deb10
                    command: executable/path.sh
                    interval: 30

        "#;
        let nodes = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }

    #[test]
    #[should_panic]
    fn condition_vm_doesnt_exist_under_infrastructure() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            infrastructure:
                deb10: 1
            conditions:
                condition-1:
                    vm-name: win10
                    command: executable/path.sh
                    interval: 30
                    
        "#;
        let nodes = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }
}
