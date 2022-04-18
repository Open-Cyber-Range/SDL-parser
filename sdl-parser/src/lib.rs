mod library_item;
mod node;

use anyhow::Result;
use chrono::{DateTime, Utc};
pub use library_item::{generate_package_list, LibraryItem};
use node::NodeMap;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub infrastructure: Option<NodeMap>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(
        alias = "Scenario",
        alias = "SCENARIO",
        deserialize_with = "deserialize_struct_case_insensitive"
    )]
    pub scenario: Scenario,
}

pub fn parse_sdl(sdl_string: &str) -> Result<Schema> {
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
        let start_time =
            DateTime::from(DateTime::parse_from_rfc3339("2022-01-20T13:00:00Z").unwrap());
        let end_time =
            DateTime::from(DateTime::parse_from_rfc3339("2022-01-20T23:00:00Z").unwrap());
        let scenario = Scenario {
            name: scenario_name.clone(),
            description: None,
            start: start_time,
            end: end_time,
            infrastructure: None,
        };
        let expected_schema = Schema { scenario };

        assert_eq!(parsed_schema, expected_schema);
        assert_eq!(parsed_schema.scenario.name, scenario_name);
        assert_eq!(parsed_schema.scenario.start, start_time);
        assert_eq!(parsed_schema.scenario.end, end_time);
        assert_eq!(parsed_schema.scenario.description, None);
    }

    #[test]
    fn includes_a_list_of_nodes() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            infrastructure:
                win10:
                    type: VM
                    description: win-10-description
                    source:
                        template: windows10
                    flavor:
                        ram: 4gb
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    flavor:
                        ram: 2gb
                        cpu: 1
        "#;
        let parsed_schema = super::parse_sdl(sdl).unwrap();
        assert!(parsed_schema.scenario.infrastructure.is_some());
        let node_map = parsed_schema.scenario.infrastructure.unwrap();
        assert_eq!(node_map.values().len(), 2);
    }

    #[test]
    fn sdl_keys_are_valid_in_lowercase_uppercase_capitalized() {
        let sdl = r#"
        scenario:
            NAME: test-scenario
            Description: some-description
            start: 2022-01-20T13:00:00Z
            End: 2022-01-20T23:00:00Z
            Infrastructure:
                Win10:
                    TYPE: VM
                    Description: win-10-description
                    Source:
                        Template: windows10
                        Package: 
                            Name: windows10
                            Version: '*'
                    Flavor:
                        Ram: 4gb
                        Cpu: 2
                    Policy:
                        Type: network
                        Rule:
                            Direction: Ingress
                            Description: some-description
                            Allowed_Address:
                                - some-ipv4
                                - some-address
                                - something-or-other
                            Port: 8080
                    Dependencies:
                        - First-dependency
                        - second-dependency
                        - third-dependency
        "#;
        let parsed_schema = super::parse_sdl(sdl).unwrap();
        insta::assert_debug_snapshot!(parsed_schema);
    }
}
