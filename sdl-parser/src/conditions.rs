use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(rename = "vm-name", alias = "Vm-name", alias = "VM-NAME")]
    pub vm_name: String,
    #[serde(default, alias = "Command", alias = "COMMAND")]
    pub command: Option<String>,
    #[serde(default, alias = "Interval", alias = "INTERVAL")]
    pub interval: Option<u32>,
    #[serde(default, alias = "Library", alias = "LIBRARY")]
    pub library: Option<String>,
}

pub type ConditionMap = HashMap<String, Condition>;

#[cfg(test)]
mod tests {
    use super::*;

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
            vm-name: green-evelation-machine
            library: digital-library-package      
        "#;
        let condition = serde_yaml::from_str::<Condition>(sdl).unwrap();
        insta::assert_debug_snapshot!(condition);
    }

    #[test]
    fn condition_list_is_parsed() {
        let sdl = r#"
            condition-1:
                vm-name: windows-10
                command: executable/path.sh
                interval: 30
            condition-2:
                vm-name: green-evelation-machine
                library: digital-library-package
            condition-3:
                vm-name: green-evelation-machine
                command: executable/path.sh
                interval: 30
        "#;
        let conditions = serde_yaml::from_str::<ConditionMap>(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }
}
