use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{condition::Condition, helpers::Connection, inject::Inject, Formalize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    #[serde(default, alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(alias = "Time", alias = "TIME")]
    pub time: Option<f32>,
    #[serde(alias = "Conditions", alias = "CONDITIONS")]
    pub conditions: Option<Vec<String>>,
    #[serde(alias = "Injects", alias = "INJECTS")]
    pub injects: Vec<String>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

pub type Events = HashMap<String, Event>;

impl Formalize for Event {
    fn formalize(&mut self) -> Result<()> {
        if self.injects.is_empty() {
            return Err(anyhow!("Event must have have at least one Inject"));
        }
        if let Some(time) = self.time {
            if !(0.0..=1.0).contains(&time) {
                return Err(anyhow!("Time must have a float value between 0 and 1"));
            }
        }
        Ok(())
    }
}
impl Connection<Condition> for (&String, &Event) {
    fn validate_connections(&self, potential_condition_names: &Option<Vec<String>>) -> Result<()> {
        if self.1.conditions.is_some() && potential_condition_names.is_none() {
            return Err(anyhow!(
                "Conditions defined for Event {} but none found under Scenario",
                self.0
            ));
        }

        if let Some(required_conditions) = &self.1.conditions {
            if let Some(condition_names) = potential_condition_names {
                for event_condition_name in required_conditions {
                    if !condition_names.contains(event_condition_name) {
                        return Err(anyhow!(
                            "Condition {event_condition_name} not found under Scenario"
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Connection<Inject> for (&String, &Event) {
    fn validate_connections(&self, potential_inject_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(inject_names) = potential_inject_names {
            for event_inject_name in &self.1.injects {
                if !inject_names.contains(event_inject_name) {
                    return Err(anyhow!(
                        "Inject {event_inject_name} not found under Scenario"
                    ));
                }
            }
        } else {
            return Err(anyhow!(
                "Inject list is empty under Scenario, but having an Event requires an Inject"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_events() {
        let sdl = r#"
            name: test-scenario
            description: some description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
            capabilities:
                capability-1:
                    description: "Can defend against Dirty Cow"
                    condition: condition-1
                capability-2:
                    description: "Can defend against Dirty Cow"
                    condition: condition-1
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
    fn parses_single_event() {
        let event = r#"
            time: 0.2345678
            conditions:
                - condition-1
            injects:
                - my-cool-inject
      "#;
        serde_yaml::from_str::<Event>(event).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_incorrect_time_value() {
        let event = r#"
            time: 600
            conditions:
                - condition-1
            injects:
                - my-cool-inject
      "#;
        serde_yaml::from_str::<Event>(event)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_scenario_condition() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                conditions:
                    condition-3000:
                        source: digital-library-package
                capabilities:
                    capability-1:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
                    capability-2:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
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

    #[test]
    #[should_panic]
    fn fails_on_missing_conditions() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                capabilities:
                    capability-1:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
                    capability-2:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
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
