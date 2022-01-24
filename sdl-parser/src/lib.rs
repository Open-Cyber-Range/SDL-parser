use anyhow::Result;
use chrono::{ DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Scenario { 
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema { 
    pub scenario: Scenario
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
        let result = super::parse_sdl(minimal_sdl).unwrap();
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
        };

        assert_eq!(result, expected_schema);
        assert_eq!(result.scenario.name, scenario_name.clone());
        assert_eq!(result.scenario.start, start_time.clone());
        assert_eq!(result.scenario.end, end_time.clone());
    }
}
