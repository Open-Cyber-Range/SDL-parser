use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Vulnerability {
    description: String,
}

pub trait VulnerabilityConnection {
    fn valid_vulnerabilities(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()>;
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

    #[test]
    fn parses_scenario_with_vulnerability_in_node() {
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
            nodes:
                win-10:
                    type: VM
                    resources:
                        ram: 2 gib
                        cpu: 2
                    source: windows10
                    vulnerabilities:
                        - vuln-2
                        - vuln-1
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn missing_vulnerability_for_node() {
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
            nodes:
                win-10:
                    type: VM
                    resources:
                        ram: 2 gib
                        cpu: 2
                    source: windows10
                    vulnerabilities:
                        - vuln-4
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn parses_vulnerability_for_feature() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-less-cool-feature:
                    source:
                        name: cool-config
                        version: 1.0.0
                    vulnerabilities:
                        - vuln-1
                        - vuln-2
            vulnerabilities:
                vuln-1:
                    description: some-description
                vuln-2:
                    description: some-description
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn missing_vulnerability_for_feature() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-less-cool-feature:
                    source:
                        name: cool-config
                        version: 1.0.0
                    vulnerabilities:
                        - vuln-4
            vulnerabilities:
                vuln-1:
                    description: some-description
                vuln-2:
                    description: some-description
            nodes:
                win-10:
                    type: VM
                    resources:
                        ram: 2 gib
                        cpu: 2
                    source: windows10
                    vulnerabilities:
                        - vuln-4
        "#;
        parse_sdl(sdl).unwrap();
    }
}
