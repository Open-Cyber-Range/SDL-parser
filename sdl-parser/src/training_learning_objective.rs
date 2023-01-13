use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{capability::Capability, evaluation::Evaluation, helpers::Connection};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct TrainingLearningObjective {
    #[serde(alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(alias = "Evaluation", alias = "EVALUATION")]
    pub evaluation: String,
    #[serde(alias = "Capabilities", alias = "CAPABILITIES")]
    pub capabilities: Option<Vec<String>>,
}

pub type TrainingLearningObjectives = HashMap<String, TrainingLearningObjective>;

impl Connection<Evaluation> for (&String, &TrainingLearningObjective) {
    fn validate_connections(&self, potential_evaluation_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(existing_evaluation_names) = potential_evaluation_names {
            if !existing_evaluation_names.contains(&self.1.evaluation) {
                return Err(anyhow!(
                    "Evaluation {} not found under scenario, but used by {} TLO",
                    self.1.evaluation,
                    self.0
                ));
            }
        } else {
            return Err(anyhow!(
                "Evaluation list is empty under scenario, but TLO: {} is declared",
                self.0
            ));
        }

        Ok(())
    }
}

impl Connection<Capability> for (&String, &TrainingLearningObjective) {
    fn validate_connections(&self, potential_capability_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(required_capabilities) = &self.1.capabilities {
            if let Some(existing_capability_names) = potential_capability_names {
                for required_capability in required_capabilities.iter() {
                    if !existing_capability_names.contains(required_capability) {
                        return Err(anyhow!(
                            "Capability {} not found under scenario",
                            required_capability
                        ));
                    }
                }
            } else if !required_capabilities.is_empty() {
                return Err(anyhow!(
                    "Capability list is empty under scenario, but training learning objective {} has capabilities",
                    self.0
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_tlos() {
        let sdl = r#"
            name: test-scenario
            description: some description
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
        "#;
        let tlos = parse_sdl(sdl).unwrap().tlos;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(tlos);
        });
    }

    #[test]
    fn parses_training_learning_objective() {
        let tlo_string = r#"
          name: test-training-learning-objective
          description: some description
          evaluation: evaluation-2
          capabilities:
            - capability-1
            - capability-2
        "#;
        serde_yaml::from_str::<TrainingLearningObjective>(tlo_string).unwrap();
    }
}
