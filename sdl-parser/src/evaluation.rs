use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{helpers::Connection, metric::Metric, Formalize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct MinScore {
    pub absolute: Option<u32>,
    pub percentage: Option<u32>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HelperScore {
    MinScore(MinScore),
    ShortMinScore(u32),
}

impl From<HelperScore> for MinScore {
    fn from(helper_source: HelperScore) -> Self {
        match helper_source {
            HelperScore::MinScore(score) => score,
            HelperScore::ShortMinScore(source) => MinScore {
                percentage: Some(source),
                absolute: None,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Evaluation {
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(alias = "Metrics", alias = "METRICS")]
    pub metrics: Vec<String>,
    #[serde(
        default,
        rename = "min-score",
        alias = "Min-score",
        alias = "MIN-SCORE",
        skip_serializing
    )]
    pub _helper_min_score: Option<HelperScore>,
    #[serde(rename = "min-score", default, skip_deserializing)]
    pub min_score: Option<MinScore>,
}

impl Connection<Metric> for (&String, &Evaluation) {
    fn validate_connections(&self, potential_metric_names: &Option<Vec<String>>) -> Result<()> {
        if let Some(metric_names) = potential_metric_names {
            for required_name in &self.1.metrics {
                if !metric_names.contains(required_name) {
                    return Err(anyhow::anyhow!(
                        "Metric {} not found for elevation: {}",
                        required_name,
                        self.0
                    ));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "No metrics found under scenario, but elevation {} exists",
                self.0
            ));
        }
        Ok(())
    }
}

pub type Evaluations = HashMap<String, Evaluation>;

impl Formalize for Evaluation {
    fn formalize(&mut self) -> Result<()> {
        if let Some(helper_min_score) = &self._helper_min_score {
            self.min_score = Some(helper_min_score.to_owned().into());
        } else {
            return Err(anyhow!("No min-score found for evaluation"));
        }
        if let Some(score) = &self.min_score {
            if score.absolute.is_some() && score.percentage.is_some() {
                return Err(anyhow!("Min-score can only have one value"));
            }
        }
        if self.metrics.is_empty() {
            return Err(anyhow!("Evaluation must have at least one metric"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn parses_sdl_with_evaluation() {
        let sdl = r#"
        scenario:
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
            evaluations:
                evaluation-1:
                    description: some description
                    metrics:
                        - metric-1
                        - metric-2
                    min-score: 50
        "#;
        let evaluations = parse_sdl(sdl).unwrap().scenario.evaluations;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(evaluations);
        });
    }

    #[test]
    #[should_panic]
    fn fails_with_missing_metric() {
        let sdl = r#"
        scenario:
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
            evaluations:
                evaluation-1:
                    description: some description
                    metrics:
                        - metric-1
                        - metric-2
                    min-score: 50
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn parses_shorthand_evaluation() {
        let evaluation_string = r#"
          description: some-description
          metrics:
            - metric-1
            - metric-2
          min-score: 50
        "#;
        let mut evaluation: Evaluation = serde_yaml::from_str(evaluation_string).unwrap();
        assert!(evaluation.formalize().is_ok());
    }

    #[test]
    fn parses_longhand_evaluation() {
        let evaluation_string = r#"
        description: some-description
        metrics:
          - metric-1
          - metric-2
        min-score:
          absolute: 50
      "#;
        let mut evaluation: Evaluation = serde_yaml::from_str(evaluation_string).unwrap();
        assert!(evaluation.formalize().is_ok());
    }

    #[test]
    fn fails_to_parse_evaluation_with_both_scores() {
        let evaluation_string = r#"
          description: some-description
          metrics:
            - metric-1
            - metric-2
          min-score:
            absolute: 50
            percentage: 60
        "#;
        let mut evaluation: Evaluation = serde_yaml::from_str(evaluation_string).unwrap();
        assert!(evaluation.formalize().is_err());
    }
}
