use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{evaluation::Evaluation, helpers::Connection, Formalize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum MetricType {
    #[serde(alias = "manual", alias = "MANUAL")]
    Manual,
    #[serde(alias = "conditional", alias = "CONDITIONAL")]
    Conditional,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Metric {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub metric_type: MetricType,
    #[serde(alias = "Artifact", alias = "ARTIFACT")]
    pub artifact: Option<bool>,
    #[serde(alias = "max-score", alias = "MAX-SCORE")]
    pub max_score: u32,
    #[serde(alias = "condition", alias = "CONDITION")]
    pub condition: Option<String>,
}

impl Connection<Metric> for (&String, &Evaluation) {
    fn validate_connections(&self, potential_metric_names: &Option<Vec<String>>) -> Result<()> {
        println!("Validating connections for metric {}", self.0);
        println!("Potential metric names: {:?}", potential_metric_names);
        if let Some(metric_names) = potential_metric_names {
            for required_name in &self.1.metrics {
                if !metric_names.contains(required_name) {
                    return Err(anyhow::anyhow!(
                        "Metric {} not found under scenario",
                        required_name
                    ));
                }
            }
        } else {
            return Err(anyhow::anyhow!("No metrics found under scenario"));
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_metrics() {
        let sdl = r#"
        scenario:
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
        let metrics = parse_sdl(sdl).unwrap().scenario.metrics;
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
