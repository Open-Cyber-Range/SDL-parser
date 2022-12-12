use anyhow::{anyhow, Result};
use duration_str::parse;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::{event::Event, helpers::Connection, Formalize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    #[serde(
        deserialize_with = "parse_time_string_to_u64_sec",
        rename = "start-time",
        alias = "Start-time",
        alias = "START-TIME"
    )]
    start_time: u64,
    #[serde(
        deserialize_with = "parse_time_string_to_u64_sec",
        rename = "end-time",
        alias = "End-time",
        alias = "END-TIME"
    )]
    end_time: u64,
    #[serde(alias = "Speed", alias = "SPEED")]
    pub speed: f32,
    #[serde(alias = "Events", alias = "EVENTS")]
    pub events: Vec<String>,
}

pub type Scripts = HashMap<String, Script>;

impl Formalize for Script {
    fn formalize(&mut self) -> Result<()> {
        if self.events.is_empty() {
            return Err(anyhow::anyhow!("Script must have have at least one Event"));
        } else if self.start_time > self.end_time {
            return Err(anyhow::anyhow!("End-time must be greater than start-time"));
        }

        if self.speed.is_sign_negative() {
            return Err(anyhow::anyhow!("Speed must have a positive value"));
        }

        Ok(())
    }
}

impl Connection<Event> for (&String, &Script) {
    fn validate_connections(&self, potential_event_names: &Option<Vec<String>>) -> Result<()> {
        if potential_event_names.is_none() {
            return Err(anyhow!(
                "Script is defined but no Events declared under Scenario"
            ));
        };

        if let Some(event_names) = potential_event_names {
            for script_event_name in &self.1.events {
                if !event_names.contains(script_event_name) {
                    return Err(anyhow!(
                        "Event {script_event_name} not found under Scenario"
                    ));
                }
            }
        }

        Ok(())
    }
}

fn parse_time_string_to_u64_sec<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut string = String::deserialize(deserializer)?;

    if string.eq("0") {
        string = String::from("0sec");
    }

    string.retain(|char| !char.is_whitespace() && char != '_');

    let duration =
        parse(&string).map_err(|_| serde::de::Error::custom("failed to parse str to duration"))?;
    Ok(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_scripts() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
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
    fn parses_single_script() {
        let script = r#"
            start-time: 5h 10min 2sec
            end-time: 1 week 7d 3 hour 10 ms
            speed: 1
            events:
                - my-cool-event
      "#;
        serde_yaml::from_str::<Script>(script).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_end_time_larger_than_start_time() {
        let script = r#"
            start-time: 1 year 5h 10min 2sec
            end-time: 1 week 7d 3 hour 10 ms
            speed: 1
            events:
                - my-cool-event
      "#;
        serde_yaml::from_str::<Script>(script)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    fn parses_zero_without_unit() {
        let script = r#"
            start-time: 0
            end-time: 1 week 7d 3 hour 10 ms
            speed: 1
            events:
                - my-cool-event
      "#;
        serde_yaml::from_str::<Script>(script)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    fn parses_underscore_formatted_numbers() {
        let script = r#"
            start-time: _1_0__0__ min
            end-time: 1_000_000s
            speed: 1
            events:
                - my-cool-event
      "#;

        let script = serde_yaml::from_str::<Script>(script).unwrap();

        assert_eq!(script.start_time, 6000);
        assert_eq!(script.end_time, 1000000);
    }

    #[test]
    #[should_panic]
    fn fails_on_negative_speed_value() {
        let script = r#"
            start-time: 0
            end-time: 3 hour
            speed: -1.234
            events:
                - my-cool-event
      "#;
        serde_yaml::from_str::<Script>(script)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_event_not_defined_for_script() {
        let sdl = r#"
            scenario:
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
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
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_event_for_script() {
        let sdl = r#"
            scenario:
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
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
                    my-embarrassing-event:
                        time: 0.2345678
                        conditions:
                            - condition-1
                        injects:
                            - my-cool-inject
            "#;
        parse_sdl(sdl).unwrap();
    }
}
