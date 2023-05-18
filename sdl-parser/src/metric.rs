use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{condition::Condition, helpers::Connection, Formalize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MetricType {
    #[serde(alias = "manual", alias = "MANUAL")]
    Manual,
    #[serde(alias = "conditional", alias = "CONDITIONAL")]
    Conditional,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Metric {
    #[serde(default, alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub metric_type: MetricType,
    #[serde(alias = "Artifact", alias = "ARTIFACT")]
    pub artifact: Option<bool>,
    #[serde(alias = "max-score", alias = "MAX-SCORE")]
    pub max_score: u32,
    #[serde(alias = "condition", alias = "CONDITION")]
    pub condition: Option<String>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

pub type Metrics = HashMap<String, Metric>;

impl Formalize for Metric {
    fn formalize(&mut self) -> Result<()> {
        if self.max_score == 0 {
            return Err(anyhow!("Metric max-score cannot be 0"));
        }
        match self.metric_type {
            MetricType::Manual => {
                if self.condition.is_some() {
                    return Err(anyhow!("Manual metric cannot have a condition"));
                }
            }
            MetricType::Conditional => {
                if self.condition.is_none() {
                    return Err(anyhow!("Conditional metric must have a condition"));
                }
                if self.artifact.is_some() {
                    return Err(anyhow!("Conditional metric cannot have an artifact"));
                }
            }
        }
        Ok(())
    }
}

impl Connection<Condition> for (&String, &Metric) {
    fn validate_connections(&self, potential_condition_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(condition_names) = potential_condition_names {
            if let Some(condition) = &self.1.condition {
                if !condition_names.contains(condition) {
                    return Err(anyhow::anyhow!(
                        "Condition {} not found under scenario",
                        condition
                    ));
                }
            }
        } else if self.1.condition.is_some() {
            return Err(anyhow::anyhow!(
                "Condition {} not found under scenario",
                self.1.condition.as_ref().unwrap()
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
    fn parses_sdl_with_metrics() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            metrics:
                metric-1:
                    type: MANUAL
                    artifact: true
                    max-score: 10
                metric-2:
                    type: CONDITIONAL
                    max-score: 10
                    condition: condition-1
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
        "#;
        let metrics = parse_sdl(sdl).unwrap().metrics;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(metrics);
        });
    }

    #[test]
    fn parses_manual_metric() {
        let metric_string = r#"
          type: MANUAL
          artifact: true
          max-score: 10
        "#;
        let mut metric: Metric = serde_yaml::from_str(metric_string).unwrap();
        assert!(metric.formalize().is_ok());
    }

    #[test]
    fn fails_manual_metric_with_condition() {
        let metric_string = r#"
          type: MANUAL
          artifact: true
          max-score: 10
          condition: some-condition
        "#;
        let mut metric: Metric = serde_yaml::from_str(metric_string).unwrap();
        assert!(metric.formalize().is_err());
    }

    #[test]
    fn parses_conditional_metric() {
        let metric_string = r#"
          type: CONDITIONAL
          max-score: 10
          condition: some-condition
        "#;
        let mut metric: Metric = serde_yaml::from_str(metric_string).unwrap();
        assert!(metric.formalize().is_ok());
    }

    #[test]
    fn fails_conditional_metric_with_artifact() {
        let metric_string = r#"
          type: CONDITIONAL
          artifact: true
          max-score: 10
          condition: some-condition
        "#;
        let mut metric: Metric = serde_yaml::from_str(metric_string).unwrap();
        assert!(metric.formalize().is_err());
    }

    #[test]
    fn fails_conditional_metric_without_artifact() {
        let metric_string = r#"
          type: CONDITIONAL
          max-score: 10
        "#;
        let mut metric: Metric = serde_yaml::from_str(metric_string).unwrap();
        assert!(metric.formalize().is_err());
    }
}
