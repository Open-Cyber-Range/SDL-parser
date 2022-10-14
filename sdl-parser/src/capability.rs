use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{condition::Condition, helpers::Connection, vulnerability::Vulnerability, Formalize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Capability {
    #[serde(default, alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(default, alias = "Condition", alias = "CONDITION")]
    pub condition: String,
    #[serde(default, alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Option<Vec<String>>,
}

impl Formalize for Capability {
    fn formalize(&mut self) -> Result<()> {
        if let Some(vulnerabilities) = &self.vulnerabilities {
            if vulnerabilities.is_empty() {
                return Err(anyhow!("When vulnerabilities is declared, capability must have at least one vulnerability"));
            }
        }
        Ok(())
    }
}

pub type Capabilities = HashMap<String, Capability>;

impl Connection<Vulnerability> for (&String, &Capability) {
    fn validate_connections(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()> {
        if let Some(capability_vulnerabilities) = &self.1.vulnerabilities {
            if let Some(vulnerabilities) = potential_vulnerability_names {
                for capability_vulnerability in capability_vulnerabilities.iter() {
                    if !vulnerabilities.contains(capability_vulnerability) {
                        return Err(anyhow!(
                            "Vulnerability {} not found under scenario",
                            capability_vulnerability
                        ));
                    }
                }
            } else if !capability_vulnerabilities.is_empty() {
                return Err(anyhow!(
                "Vulnerability list is empty under scenario, but capability {} has vulnerabilities",
                self.0
            ));
            }
        }
        Ok(())
    }
}

impl Connection<Condition> for (&String, &Capability) {
    fn validate_connections(&self, potential_condition_names: &Option<Vec<String>>) -> Result<()> {
        let condition = &self.1.condition;

        if let Some(condition_names) = potential_condition_names {
            if !condition_names.contains(condition) {
                return Err(anyhow!("Condition {} not found under scenario", condition));
            }
        } else {
            return Err(anyhow!("Condition list is empty under scenario, but having a capability requires a condition"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn capability_is_parsed() {
        let sdl = r#"
          description: "Can defend against Dirty Cow"
          condition: condition-4
          vulnerabilities:
            - vulnerability-1
            - vulnerability-2
        "#;
        serde_yaml::from_str::<Capability>(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn capability_with_zero_vulnerabilities_fails_formalization() {
        let sdl = r#"
          description: "Can defend against Dirty Cow"
          condition: condition-4
          vulnerabilities: {}
        "#;
        let mut capability = serde_yaml::from_str::<Capability>(sdl).unwrap();
        capability.formalize().unwrap();
    }

    #[test]
    fn can_parse_capabilities_in_sdl() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            vulnerabilities:
              vulnerability-1:
                description: some-description
              vulnerability-2:
                description: some-description
            conditions:
              condition-1:
                command: executable/path.sh
                interval: 30
                source: digital-library-package
              condition-2:
                source: digital-library-package
            capabilities:
              capability-1:
                description: "Can execute Dirty Cow"
                condition: condition-1
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
              capability-2:
                description: "Can defend against Dirty Cow"
                condition: condition-2
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
        "#;
        let capabilities = parse_sdl(sdl).unwrap().scenario.capabilities;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(capabilities);
        });
    }

    #[test]
    #[should_panic]
    fn fails_parsing_when_missing_condition() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            vulnerabilities:
              vulnerability-1:
                description: some-description
              vulnerability-2:
                description: some-description
            conditions:
              condition-1:
                command: executable/path.sh
                interval: 30
                source: digital-library-package
              condition-2:
                source: digital-library-package
            capabilities:
              capability-1:
                description: "Can execute Dirty Cow"
                condition: condition-1
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
              capability-2:
                description: "Can defend against Dirty Cow"
                condition: condition-3
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_parsing_when_missing_vulnerability() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            vulnerabilities:
              vulnerability-1:
                description: some-description
              vulnerability-2:
                description: some-description
            conditions:
              condition-1:
                command: executable/path.sh
                interval: 30
                source: digital-library-package
              condition-2:
                source: digital-library-package
            capabilities:
              capability-1:
                description: "Can execute Dirty Cow"
                condition: condition-1
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
                  - vulnerability-4
              capability-2:
                description: "Can defend against Dirty Cow"
                condition: condition-2
                vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
        "#;
        parse_sdl(sdl).unwrap();
    }
}
