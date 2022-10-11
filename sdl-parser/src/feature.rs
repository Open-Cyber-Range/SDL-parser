use crate::{
    common::{HelperSource, Source},
    vulnerabilities::VulnerabilityConnection,
    Formalize,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    #[serde(
        default,
        rename = "source",
        alias = "Source",
        alias = "SOURCE",
        skip_serializing
    )]
    _source_helper: Option<HelperSource>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
    pub dependencies: Option<Vec<String>>,
    pub vulnerabilities: Option<Vec<String>>,
}

impl VulnerabilityConnection for (&String, &Feature) {
    fn valid_vulnerabilities(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()> {
        if let Some(node_vulnerabilities) = &self.1.vulnerabilities {
            if let Some(vulnerabilities) = potential_vulnerability_names {
                for node_vulnerability in node_vulnerabilities.iter() {
                    if !vulnerabilities.contains(node_vulnerability) {
                        return Err(anyhow!(
                            "Vulnerability {} not found under scenario",
                            node_vulnerability
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Vulnerability list is empty under scenario, but feature {} has vulnerabilities",
                    self.0
                ));
            }
        }
        Ok(())
    }
}

pub type Features = HashMap<String, Feature>;

impl Formalize for Feature {
    fn formalize(&mut self) -> Result<()> {
        match &mut self._source_helper.take() {
            Some(HelperSource::Source(source)) => {
                self.source = Some(source.to_owned());
                Ok(())
            }
            Some(HelperSource::ShortSource(source)) => {
                self.source = Some(Source {
                    name: source.to_owned(),
                    version: "*".to_string(),
                });
                Ok(())
            }
            None => Err(anyhow!("No source found for Feature")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_sdl;

    #[test]
    fn feature_source_fields_are_mapped_correctly() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-cool-feature:
                    source: some-service
                my-cool-feature-config:
                    source:
                        name: cool-config
                        version: 1.0.0
        "#;
        let features = parse_sdl(sdl).unwrap().scenario.features;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(features);
        });
    }

    #[test]
    fn feature_source_longhand_is_parsed() {
        let longhand_source = r#"
            source:
                name: artifact-name
                version: 1.2.3
        "#;
        let feature = serde_yaml::from_str::<Feature>(longhand_source).unwrap();
        insta::assert_debug_snapshot!(feature);
    }
    #[test]
    fn feature_source_shorthand_is_parsed() {
        let shorthand_source = r#"
            source: artifact-name
        "#;
        let feature = serde_yaml::from_str::<Feature>(shorthand_source).unwrap();
        insta::assert_debug_snapshot!(feature);
    }

    #[test]
    fn feature_includes_dependencies() {
        let feature_sdl = r#"
            source: some-service
            dependencies:
                - some-virtual-machine
                - some-switch
                - something-else
        "#;
        let feature = serde_yaml::from_str::<Feature>(feature_sdl).unwrap();
        insta::assert_debug_snapshot!(feature);
    }

    #[test]
    fn cyclic_feature_dependency_is_detected() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-cool-feature:
                    source: some-service
                    dependencies: 
                        - my-less-cool-feature
                my-less-cool-feature:
                    source:
                        name: cool-config
                        version: 1.0.0
                    dependencies: 
                        - my-cool-feature
        "#;
        let features = parse_sdl(sdl);
        assert!(features.is_err());
        assert_eq!(
            features.err().unwrap().to_string(),
            "Cyclic dependency detected"
        );
    }

    #[test]
    fn feature_cyclic_self_dependency_is_detected() {
        let sdl = r#"
        scenario:
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-cool-feature:
                    source: some-service
                    dependencies: 
                        - my-cool-feature
        "#;
        let features = parse_sdl(sdl);
        assert!(features.is_err());
        assert_eq!(
            features.err().unwrap().to_string(),
            "Cyclic dependency detected"
        );
    }
}
