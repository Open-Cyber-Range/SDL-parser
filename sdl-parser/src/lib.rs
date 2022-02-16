mod node;

use anyhow::Result;
use chrono::{ DateTime, Utc};
use node::NodeMap;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Scenario { 
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema { 
    pub scenario: Scenario,
    pub nodes: Option<NodeMap>
}

pub fn parse_sdl (sdl_string: &str) -> Result<Schema> {
    Ok(serde_yaml::from_str(sdl_string)?)
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::DateTime;

    #[test]
    fn can_parse_minimal_sdl() {
        let minimal_sdl = r#"
            scenario:
                name: test-scenario
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
        "#;
        let parsed_schema = super::parse_sdl(minimal_sdl).unwrap();
        let scenario_name = "test-scenario".to_string();
        let start_time = DateTime::from(DateTime::parse_from_rfc3339("2022-01-20T13:00:00Z").unwrap());
        let end_time = DateTime::from(DateTime::parse_from_rfc3339("2022-01-20T23:00:00Z").unwrap());
        let scenario = Scenario {
            name: scenario_name.clone(),
            start: start_time,
            end: end_time
        };
        let expected_schema = Schema {
            scenario,
            nodes: None
        };

        assert_eq!(parsed_schema, expected_schema);
        assert_eq!(parsed_schema.scenario.name, scenario_name);
        assert_eq!(parsed_schema.scenario.start, start_time);
        assert_eq!(parsed_schema.scenario.end, end_time);
    }

    #[test]
    fn includes_a_list_of_nodes() {
        let sdl = r#"
            scenario:
                name: test-scenario
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    template: windows10
                    flavor:
                        ram: 4gb
                        cpu: 2
                deb10:
                    type: VM
                    template: debian10
                    flavor:
                        ram: 2gb
                        cpu: 1
        "#;
        let parsed_schema = super::parse_sdl(sdl).unwrap();

        assert!(parsed_schema.nodes.is_some());
        assert_eq!(parsed_schema.nodes.unwrap().values().len(), 2)
    }
}
