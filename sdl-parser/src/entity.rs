use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum ExerciseRole {
    White,
    Green,
    Red,
    Blue,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    #[serde(alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(alias = "Role", alias = "ROLE")]
    pub role: Option<ExerciseRole>,
    #[serde(alias = "Mission", alias = "MISSION")]
    pub mission: Option<String>,
    #[serde(alias = "Categories", alias = "CATEGORIES")]
    pub categories: Option<Vec<String>>,
    #[serde(alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Option<Vec<String>>,
    #[serde(alias = "Tlos", alias = "TLOS")]
    pub tlos: Option<Vec<String>>,
    #[serde(alias = "Entities", alias = "ENTITIES")]
    pub entities: Option<Entities>,
}

pub type Entities = HashMap<String, Entity>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_entities() {
        let sdl = r#"
      scenario:
          name: test-scenario
          description: some-description
          start: 2022-01-20T13:00:00Z
          end: 2022-01-20T23:00:00Z
          conditions:
            condition-1:
                command: executable/path.sh
                interval: 30
          metrics:
              metric-1:
                  type: MANUAL
                  artifact: true
                  max-score: 10
              metric-2:
                  type: CONDITIONAL
                  max-score: 10
                  condition: condition-1
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
          evaluations:
              evaluation-1:
                  description: some description
                  metrics:
                      - metric-1
                      - metric-2
                  min-score: 50
          capabilities:
              capability-1:
                  description: "Can defend against Dirty Cow"
                  condition: condition-1
                  vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
              capability-2:
                  description: "Can defend against Dirty Cow"
                  condition: condition-1
                  vulnerabilities:
                  - vulnerability-1
                  - vulnerability-2
          tlos:
              tlo-1:
                  description: some description
                  evaluation: evaluation-1
                  capabilities:
                      - capability-1
                      - capability-2
          entities:
              my-organization:
                  name: "My Organization"
                  description: "This is my organization"
                  role: White
                  mission: "defend"
                  categories:
                  - Foundation
                  - Organization
                  vulnerabilities:
                  - vulnerability-2
                  tlos:
                  - tlo-1
                  entities:
                  fish:
                      name: "Shark"
                      description: "This is my organization"
                      mission: "swim around"
                      categories:
                      - Animal
      "#;
        let entities = parse_sdl(sdl).unwrap().scenario.entities;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(entities);
        });
    }

    #[test]
    fn parses_single_entity() {
        let entity_yml = r#"
          name: "My Organization"
          description: "This is my organization"
          role: White
          mission: "defend"
          categories:
            - Foundation
            - Organization
          vulnerabilities:
            - vulnerability-2
          tlos:
            - tlo-1
            - tlo-2
            - tlo-3
        "#;
        serde_yaml::from_str::<Entity>(entity_yml).unwrap();
    }

    #[test]
    fn parses_nested_entity() {
        let entity_yml = r#"
          name: "My Organization"
          description: "This is my organization"
          role: White
          mission: "defend"
          categories:
            - Foundation
            - Organization
          vulnerabilities:
            - vulnerability-2
          tlos:
            - tlo-1
            - tlo-2
            - tlo-3
          entities:
            fish:
              name: "Shark"
              description: "This is my organization"
              mission: "swim around"
              categories:
                - Animal
        "#;
        serde_yaml::from_str::<Entity>(entity_yml).unwrap();
    }
}
