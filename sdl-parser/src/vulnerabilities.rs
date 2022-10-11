use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Vulnerability {
    description: String,
}

pub type Vulnerabilities = HashMap<String, Vulnerability>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn vulnerability_is_parsed() {
        let sdl = r#"
            description: some-description
        "#;
        serde_yaml::from_str::<Vulnerability>(sdl).unwrap();
    }

    #[test]
    fn vulnerability_is_parsed_in_scenario() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            vulnerabilities:
                vuln-1:
                    description: some-description
                vuln-2:
                    description: some-description
        "#;
        parse_sdl(sdl).unwrap();
    }
}
