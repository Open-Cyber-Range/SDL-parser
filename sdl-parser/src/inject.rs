use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    capability::Capability,
    common::{HelperSource, Source},
    entity::Entity,
    helpers::Connection,
    training_learning_objective::TrainingLearningObjective,
    Formalize,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Inject {
    #[serde(default, alias = "Name", alias = "NAME")]
    pub name: Option<String>,
    #[serde(
        default,
        rename = "source",
        alias = "Source",
        alias = "SOURCE",
        skip_serializing
    )]
    source_helper: Option<HelperSource>,
    #[serde(default, skip_deserializing)]
    pub source: Option<Source>,
    #[serde(rename = "from-entity", alias = "From-entity", alias = "FROM-ENTITY")]
    pub from_entity: Option<String>,
    #[serde(rename = "to-entities", alias = "To-entities", alias = "TO-ENTITIES")]
    pub to_entities: Option<Vec<String>>,
    #[serde(alias = "Tlos", alias = "TLOS")]
    pub tlos: Option<Vec<String>>,
    #[serde(alias = "Capabilities", alias = "CAPABILITIES")]
    pub capabilities: Option<Vec<String>>,
    #[serde(alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
}

pub type Injects = HashMap<String, Inject>;

impl Formalize for Inject {
    fn formalize(&mut self) -> Result<()> {
        if self.from_entity.is_some() && self.to_entities.is_none() {
            return Err(anyhow!(
                "Inject must have `to-entities` declared if `from-entity` is declared"
            ));
        } else if self.from_entity.is_none() && self.to_entities.is_some() {
            return Err(anyhow!(
                "Inject must have `from-entity` declared if `to-entities` is declared"
            ));
        } else if let Some(source_helper) = &self.source_helper {
            self.source = Some(source_helper.to_owned().into());
        }
        Ok(())
    }
}

impl Connection<Entity> for (&String, &Inject) {
    fn validate_connections(&self, potential_entity_names: &Option<Vec<String>>) -> Result<()> {
        if self.1.to_entities.is_some() && potential_entity_names.is_none()
            || self.1.from_entity.is_some() && potential_entity_names.is_none()
        {
            return Err(anyhow!(
                "Inject \"{inject_name}\" has Entities but none found under Scenario",
                inject_name = self.0
            ));
        }

        let mut required_entities: Vec<String> = vec![];

        if let Some(from_entity) = self.1.clone().from_entity {
            required_entities.push(from_entity);
            if let Some(to_entities) = self.1.clone().to_entities {
                required_entities.extend_from_slice(to_entities.as_slice());
            }
        }
        for inject_entity_name in required_entities.iter() {
            if let Some(scenario_entities) = potential_entity_names {
                if !scenario_entities.contains(inject_entity_name) {
                    return Err(anyhow!(
                        "Inject \"{inject_name}\" Entity \"{inject_entity_name}\" not found under Scenario Injects", 
                        inject_name = self.0
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Connection<TrainingLearningObjective> for (&String, &Inject) {
    fn validate_connections(&self, potential_tlo_names: &Option<Vec<String>>) -> Result<()> {
        if self.1.tlos.is_some() && potential_tlo_names.is_none() {
            return Err(anyhow!(
                "Inject \"{inject_name}\" has TLOs but none found under Scenario",
                inject_name = self.0
            ));
        }

        if let Some(required_tlos) = &self.1.tlos {
            if let Some(tlo_names) = potential_tlo_names {
                for tlo_name in required_tlos {
                    if !tlo_names.contains(tlo_name) {
                        return Err(anyhow!("Inject \"{inject_name}\" TLO \"{tlo_name}\" not found under Scenario TLOs",
                        inject_name = self.0
                    ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Connection<Capability> for (&String, &Inject) {
    fn validate_connections(&self, potential_capability_names: &Option<Vec<String>>) -> Result<()> {
        if self.1.capabilities.is_some() && potential_capability_names.is_none() {
            return Err(anyhow!(
                "Inject \"{inject_name}\" has Capabilities but none found under Scenario",
                inject_name = self.0
            ));
        }

        if let Some(required_capabilities) = &self.1.capabilities {
            if let Some(scenario_capability_names) = potential_capability_names {
                for inject_capability_name in required_capabilities.iter() {
                    if !scenario_capability_names.contains(inject_capability_name) {
                        return Err(anyhow!(
                            "Inject \"{inject_name}\" Capability \"{inject_capability_name}\" not found under Scenario Capabilities",
                            inject_name = self.0
                        ));
                    }
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
    fn parses_sdl_with_injects() {
        let sdl = r#"
            name: test-scenario
            description: some description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            capabilities:
                capability-1:
                    description: "Can defend against Dirty Cow"
                    condition: condition-1
                capability-2:
                    description: "Can defend against Dirty Cow"
                    condition: condition-1
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
            tlos:
                tlo-1:
                    name: fungibly leverage client-focused e-tailers
                    description: we learn to make charts of web page stats
                    evaluation: evaluation-1
                    capabilities:
                        - capability-1
                        - capability-2
            evaluations:
                evaluation-1:
                    description: some description
                    metrics:
                        - metric-1
                        - metric-2
                    min-score: 50
            entities:
                my-organization:
                    name: "My Organization"
                    description: "This is my organization"
                    role: White
                    mission: "defend"
                    categories:
                        - Foundation
                        - Organization
                red-team:
                    name: "The Red Team"
                    description: "The Red Team attempts to penetrate the target organization"
                    role: Red
                    mission: "Attack"
                blue-team:
                    name: "The Blue Team"
                    description: "They defend from attacks and respond to incidents"
                    role: Red
                    mission: "Attack"
            injects:
                my-cool-inject:
                    source: inject-package
                    from-entity: my-organization
                    to-entities:
                        - red-team
                        - blue-team
                    tlos:
                        - tlo-1
                    capabilities:
                        - capability-2
        "#;
        let injects = parse_sdl(sdl).unwrap();

        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(injects);
        });
    }

    #[test]
    fn parses_single_inject() {
        let inject = r#"
            source: inject-package
            from-entity: my-organization
            to-entities:
                - red-team
                - blue-team
            tlos:
                - tlo-1
            capabilities:
                - capability-2
      "#;
        serde_yaml::from_str::<Inject>(inject).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_to_entities_declared_but_from_entities_not_declared() {
        let inject = r#"
                source: inject-package
                to-entities:
                    - red-team
                    - blue-team
                tlos:
                    - tlo-1
                capabilities:
                    - capability-2
      "#;

        serde_yaml::from_str::<Inject>(inject)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_from_entities_declared_but_to_entities_not_declared() {
        let inject = r#"
                source: inject-package
                from-entity: gray-hats
                tlos:
                    - tlo-1
                capabilities:
                    - capability-2
      "#;

        serde_yaml::from_str::<Inject>(inject)
            .unwrap()
            .formalize()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_capabilities_not_defined_for_inject() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                injects:
                    my-cool-inject:
                        source: inject-package
                        capabilities:
                            - capability-2
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_capability() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                conditions:
                    condition-1:
                        source: digital-library-package
                capabilities:
                    capability-9999:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
                injects:
                    my-cool-inject:
                        source: inject-package
                        capabilities:
                            - capability-1
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_capabilities_not_defined_for_tlo() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                conditions:
                    condition-1:
                        command: executable/path.sh
                        interval: 30
                capabilities:
                    capability-9999:
                        description: "Can defend against Dirty Cow"
                        condition: condition-1
                tlos:
                    tlo-1:
                        name: fungibly leverage client-focused e-tailers
                        description: we learn to make charts of web page stats
                        evaluation: evaluation-1
                        capabilities:
                            - capability-1
                evaluations:
                    evaluation-1:
                        description: some description
                        metrics:
                            - metric-1
                        min-score: 50
                metrics:
                        metric-1:
                            type: MANUAL
                            artifact: true
                            max-score: 10
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_tlo_not_defined() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                evaluations:
                    evaluation-1:
                        description: some description
                        metrics:
                            - metric-1
                        min-score: 50
                metrics:
                        metric-1:
                            type: MANUAL
                            artifact: true
                            max-score: 10
                injects:
                    my-cool-inject:
                        source: inject-package
                        tlos:
                            - tlo-1
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_tlo_for_inject() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                evaluations:
                    evaluation-1:
                        description: some description
                        metrics:
                            - metric-1
                        min-score: 50
                metrics:
                        metric-1:
                            type: MANUAL
                            artifact: true
                            max-score: 10
                injects:
                    my-cool-inject:
                        source: inject-package
                        tlos:
                            - tlo-1
                tlos:
                    tlo-9999:
                        name: fungibly leverage client-focused e-tailers
                        description: we learn to make charts of web page stats
                        evaluation: evaluation-1
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_entity_not_defined_for_inject() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                injects:
                    my-cool-inject:
                        source: inject-package
                        from-entity: my-organization
                        to-entities:
                            - red-team
                            - blue-team
            "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_on_missing_entity_for_inject() {
        let sdl = r#"
                name: test-scenario
                description: some description
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                entities:
                    red-team:
                        name: "The Red Team"
                    blue-team:
                        name: "The Blue Team"
                injects:
                    my-cool-inject:
                        source: inject-package
                        from-entity: my-organization
                        to-entities:
                            - red-team
                            - blue-team
            "#;
        parse_sdl(sdl).unwrap();
    }
}
