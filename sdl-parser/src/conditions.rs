use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    common::{HelperSource, Source},
    Formalize,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
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
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                condition-2:
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
            command: executable/path.sh
            interval: 30       

        "#;
        let condition = serde_yaml::from_str::<Condition>(sdl).unwrap();
        insta::assert_debug_snapshot!(condition);
    }

    #[test]
    fn library_condition_is_parsed() {
        let sdl = r#"
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
                    conditions:
                        - condition-1
                deb-10:
                    type: VM
                    source:
                        name: debian10
                        version: '*'
                    conditions:
                        - condition-2
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                    source: digital-library-package
                condition-2:
                    source: digital-library-package

        "#;
        let schema = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }
}
