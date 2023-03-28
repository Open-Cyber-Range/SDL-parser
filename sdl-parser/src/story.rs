use crate::helpers::Connection;
use crate::Formalize;
use crate::{constants::default_clock_value, script::Script};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Story {
    #[serde(default = "default_clock_value", alias = "Clock", alias = "CLOCK")]
    pub clock: u64,
    #[serde(alias = "Scripts", alias = "SCRIPTS")]
    pub scripts: Vec<String>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

impl Story {
    pub fn new(potential_clock: Option<u64>) -> Self {
        Self {
            clock: match potential_clock {
                Some(clock) => clock,
                None => default_clock_value(),
            },
            ..Default::default()
        }
    }
}

pub type Stories = HashMap<String, Story>;

impl Formalize for Story {
    fn formalize(&mut self) -> Result<()> {
        if self.scripts.is_empty() {
            return Err(anyhow!("Story must have have at least one Script"));
        }

        if self.clock < 1 {
            return Err(anyhow!("Clock value must be at least 1"));
        }

        Ok(())
    }
}

impl Connection<Script> for (&String, &Story) {
    fn validate_connections(&self, potential_script_names: &Option<Vec<String>>) -> Result<()> {
        if potential_script_names.is_none() {
            return Err(anyhow!(
                "Story is defined but no Scripts declared under Scenario"
            ));
        };

        if let Some(script_names) = potential_script_names {
            for story_script_name in &self.1.scripts {
                if !script_names.contains(story_script_name) {
                    return Err(anyhow!(
                        "Script {story_script_name} not found under Scenario"
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_stories() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                stories: 
                    story-1: 
                        clock: 1
                        scripts: 
                            - my-cool-script
                conditions:
                    condition-1:
                        command: executable/path.sh
                        interval: 30
                        source: digital-library-package
                scripts:
                    my-cool-script:
                        start-time: 10min 2 sec
                        end-time: 1 week 1day 1h 10 ms
                        speed: 1.5
                        events:
                            - my-cool-event
                injects:
                    my-cool-inject:
                        source: inject-package
                events:
                    my-cool-event:
                        time: 0.2345678
                        conditions:
                            - condition-1
                        injects:
                            - my-cool-inject
            "#;
        let schema = parse_sdl(sdl).unwrap();

        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(schema);
        });
    }

    #[test]
    fn parses_single_story() {
        let story = r#" 
            clock: 1
            scripts: 
                - script-1
                - script-2
     
      "#;
        serde_yaml::from_str::<Story>(story).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_clock_is_zero() {
        let story = r#"
            clock: 0
            scripts: 
                - script-1
                - script-2
      "#;
        serde_yaml::from_str::<Story>(story)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    fn adds_default_clock() {
        let story = r#"
            scripts: 
                - script-1
                - script-2
      "#;

        let story = serde_yaml::from_str::<Story>(story).unwrap();

        assert_eq!(story.clock, 1);
    }

    #[test]
    #[should_panic]
    fn fails_when_scripts_is_empty() {
        let story = r#"
            clock: 1
            scripts:
      "#;
        serde_yaml::from_str::<Story>(story)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_when_no_scripts_field() {
        let story = r#"
            clock: 15
      "#;
        serde_yaml::from_str::<Story>(story)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_script_not_defined_for_story() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                stories: 
                    story-1: 
                        clock: 1
                        scripts: 
                            - script-1
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_script_for_story() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                stories: 
                    story-1: 
                        clock: 1
                        scripts: 
                            - script-1
                conditions:
                    condition-1:
                        command: executable/path.sh
                        interval: 30
                        source: digital-library-package
                scripts:
                    my-cool-script:
                        start-time: 10min 2 sec
                        end-time: 1 week 1day 1h 10 ms
                        speed: 1.5
                        events:
                            - my-cool-event
                injects:
                    my-cool-inject:
                        source: inject-package
                events:
                    my-cool-event:
                        time: 0.2345678
                        conditions:
                            - condition-1
                        injects:
                            - my-cool-inject
            "#;
        parse_sdl(sdl).unwrap();
    }
}
