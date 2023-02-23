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
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

impl Formalize for Condition {
    fn formalize(&mut self) -> Result<()> {
        if self.command.is_some() && self.interval.is_some() {
            self.source_helper = None;
        } else if let Some(source_helper) = &self.source_helper {
            self.source = Some(source_helper.to_owned().into());
        } else if self.command.is_some() && self.interval.is_none() {
            return Err(anyhow::anyhow!("No interval found for command"));
        } else if self.command.is_none() && self.interval.is_some() {
            return Err(anyhow::anyhow!(
                "No command found for use with the interval"
            ));
        } else {
            return Err(anyhow::anyhow!(
                "Command or source missing for the condition"
            ));
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
    fn conditions_are_mapped_correctly() {
        let sdl = r#"
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
        let conditions = parse_sdl(sdl).unwrap().conditions;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }

    #[test]
    fn handles_metrics_with_conditions_correctly() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                    description: This is a description for condition 1
                condition-2:
                    source: digital-library-package
                    description: This is a description for condition 2
            metrics:
                metric-1:
                    type: MANUAL
                    artifact: true
                    max-score: 10
                metric-2:
                    type: CONDITIONAL
                    max-score: 10
                    condition:  condition-2
        "#;
        let conditions = parse_sdl(sdl).unwrap().conditions;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }

    #[test]
    #[should_panic]
    fn identifies_missing_condition() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                    description: This is a description for condition 1
                condition-2:
                    source: digital-library-package
                    description: This is a description for condition 2
            metrics:
                metric-1:
                    type: MANUAL
                    artifact: true
                    max-score: 10
                metric-2:
                    type: CONDITIONAL
                    max-score: 10
                    condition:  condition-3
        "#;
        parse_sdl(sdl).unwrap();
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
    fn command_condition_is_parsed_correctly_with_both_command_and_source() {
        let sdl = r#"
        
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                    source: digital-library-package
                    description: This is a description for condition 1
        "#;
        let conditions = parse_sdl(sdl).unwrap().conditions;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }
}
