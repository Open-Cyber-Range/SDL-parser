use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{condition::Condition, helpers::Connection, vulnerability::Vulnerability, Formalize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Capability {
    #[serde(default, alias = "Name", alias = "NAME")]
    pub name: Option<String>,
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
                return Err(anyhow!(
                    "Capability requires at least one Vulnerability but none found under Scenario"
                ));
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
                for vulnerability_name in capability_vulnerabilities.iter() {
                    if !vulnerabilities.contains(vulnerability_name) {
                        return Err(anyhow!(
                            "Vulnerability \"{vulnerability_name}\" not found under Scenario Vulnerabilities",
                        ));
                    }
                }
            } else if !capability_vulnerabilities.is_empty() {
                return Err(anyhow!(
                "Capability \"{capability_name}\" has Vulnerabilities but none found under Scenario",
                capability_name = self.0
            ));
            }
        }
        Ok(())
    }
}

impl Connection<Condition> for (&String, &Capability) {
    fn validate_connections(&self, potential_condition_names: &Option<Vec<String>>) -> Result<()> {
        let condition_name = &self.1.condition;

        if let Some(condition_names) = potential_condition_names {
            if !condition_names.contains(condition_name) {
                return Err(anyhow!(
                    "Condition \"{condition_name}\" not found under Scenario Conditions"
                ));
            }
        } else {
            return Err(anyhow!(
                "Capability requires at least one Condition but none found under Scenario"
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
    #[should_panic(expected = "expected a sequence")]
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
            name: test-scenario
            description: some-description
            vulnerabilities:
              vulnerability-1:
                name: Some other vulnerability
                description: some-description
                technical: false
                class: CWE-1343
              vulnerability-2:
                name: Some vulnerability
                description: some-description
                technical: false
                class: CWE-1341
            conditions:
              condition-1:
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
        let capabilities = parse_sdl(sdl).unwrap().capabilities;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(capabilities);
        });
    }

    #[test]
    #[should_panic(expected = "Condition \"condition-3\" not found under Scenario Conditions")]
    fn fails_parsing_when_missing_condition() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            vulnerabilities:
              vulnerability-1:
                name: some-vulnerability
                description: some-description
                technical: false
                class: CWE-1343
              vulnerability-2:
                name: some-vulnerability
                description: some-description
                technical: false
                class: CWE-1343
            conditions:
              condition-1:
                command: executable/path.sh
                interval: 30
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
    #[should_panic(expected = "Vulnerability \"vulnerability-4\" not found under Scenario Vulnerabilities")]
    fn fails_parsing_when_missing_vulnerability() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            vulnerabilities:
              vulnerability-1:
                name: some-vulnerability
                description: some-description
                technical: false
                class: CWE-1343
              vulnerability-2:
                name: some-vulnerability
                description: some-description
                technical: false
                class: CWE-1343
            conditions:
              condition-1:
                command: executable/path.sh
                interval: 30
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
