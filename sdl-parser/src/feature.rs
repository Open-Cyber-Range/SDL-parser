use crate::node::{Source, SourceArray};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum FeatureType {
    Service,
    Configuration,
    Artifact,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub feature_type: FeatureType,
    #[serde(
        default,
        rename = "source",
        alias = "Source",
        alias = "SOURCE",
        skip_serializing
    )]
    _source_helper: Option<SourceArray>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
    pub dependencies: Option<Vec<String>>,
}

pub type FeatureMap = HashMap<String, Feature>;

impl Feature {
    pub fn map_source(&mut self) {
        match &mut self._source_helper.take() {
            Some(SourceArray::Source(source)) => {
                self.source = Some(source.to_owned());
            }
            Some(SourceArray::ShortSource(source)) => {
                self.source = Some(Source {
                    name: source.to_owned(),
                    version: "*".to_string(),
                });
            }
            None => {}
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
                    type: Service
                    source: some-service
                my-cool-feature-config:
                    type: Configuration
                    source:
                        name: cool-config
                        version: 1.0.0
        "#;
        let features = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(features);
        });
    }

    #[test]
    fn feature_source_longhand_is_parsed() {
        let longhand_source = r#"
            type: Artifact
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
            type: Artifact
            source: artifact-name
        "#;
        let feature = serde_yaml::from_str::<Feature>(shorthand_source).unwrap();
        insta::assert_debug_snapshot!(feature);
    }

    #[test]
    fn feature_includes_dependencies() {
        let feature_sdl = r#"
            type: Service
            source: some-service
            dependencies:
                - some-virtual-machine
                - some-switch
                - something-else
        "#;
        let feature = serde_yaml::from_str::<Feature>(feature_sdl).unwrap();
        insta::assert_debug_snapshot!(feature);
    }
}
