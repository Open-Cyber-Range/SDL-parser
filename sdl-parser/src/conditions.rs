use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    common::{get_source, Source, SourceArray},
    infrastructure::Infrastructure,
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
    _source_helper: Option<SourceArray>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
}

impl Condition {
    pub fn map_source(&mut self) {
        self.source = get_source(self._source_helper.take());
    }

    pub fn check_vm_count(&self, infrastructure: Infrastructure) {
        for (node_name, node) in infrastructure.iter() {
            match self.vm_name {
                node_name => {
                    if node.count != 1 {
                        Err("Node count not 1");
                    }
                }
            }
        }
    }
}

pub type ConditionMap = HashMap<String, Condition>;

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

    // #[test]
    // fn condition_list_is_parsed() {
    //     let sdl = r#"
    //         condition-1:
    //             vm-name: windows-10
    //             command: executable/path.sh
    //             interval: 30
    //         condition-2:
    //             vm-name: green-evaluation-machine
    //             source: digital-library-package
    //         condition-3:
    //             vm-name: green-evaluation-machine
    //             command: executable/path.sh
    //             interval: 30
    //     "#;
    //     let conditions = serde_yaml::from_str::<ConditionMap>(sdl).unwrap();
    //     insta::with_settings!({sort_maps => true}, {
    //             insta::assert_yaml_snapshot!(conditions);
    //     });
    // }
}
