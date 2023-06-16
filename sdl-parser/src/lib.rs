pub mod capability;
pub mod common;
pub mod condition;
mod constants;
pub mod entity;
pub mod evaluation;
pub mod event;
pub mod feature;
pub mod goal;
mod helpers;
pub mod infrastructure;
pub mod inject;
mod library_item;
pub mod metric;
pub mod node;
pub mod script;
pub mod story;
pub mod training_learning_objective;
pub mod vulnerability;

use crate::helpers::Connection;
use anyhow::{anyhow, Ok, Result};
use capability::{Capabilities, Capability};
use chrono::{DateTime, Utc};
use condition::{Condition, Conditions};
use constants::MAX_LONG_NAME;
use depper::{Dependencies, DependenciesBuilder};
use entity::{Entities, Entity, Flatten};
use evaluation::{Evaluation, Evaluations};
use event::{Event, Events};
use feature::{Feature, Features};
use goal::Goals;
use infrastructure::{Infrastructure, InfrastructureHelper};
use inject::{Inject, Injects};
pub use library_item::LibraryItem;
use metric::{Metric, Metrics};
use node::{Node, NodeType, Nodes};
use script::{Script, Scripts};
use serde::{Deserialize, Serialize};
use story::Stories;
use training_learning_objective::{TrainingLearningObjective, TrainingLearningObjectives};
use vulnerability::{Vulnerabilities, Vulnerability};

pub trait Formalize {
    fn formalize(&mut self) -> Result<()>;
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    #[serde(alias = "Name", alias = "NAME")]
    pub name: String,
    #[serde(default, alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(alias = "Start", alias = "START")]
    pub start: DateTime<Utc>,
    #[serde(alias = "End", alias = "END")]
    pub end: DateTime<Utc>,
    #[serde(alias = "Nodes", alias = "NODES")]
    pub nodes: Option<Nodes>,
    #[serde(alias = "Features", alias = "FEATURES")]
    pub features: Option<Features>,
    #[serde(
        default,
        rename = "infrastructure",
        skip_serializing,
        alias = "Infrastructure",
        alias = "INFRASTRUCTURE"
    )]
    infrastructure_helper: Option<InfrastructureHelper>,
    #[serde(
        default,
        skip_deserializing,
        alias = "Infrastructure",
        alias = "INFRASTRUCTURE"
    )]
    pub infrastructure: Option<Infrastructure>,
    #[serde(alias = "Conditions", alias = "CONDITIONS")]
    pub conditions: Option<Conditions>,
    #[serde(alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Option<Vulnerabilities>,
    #[serde(alias = "Capabilities", alias = "CAPABILITIES")]
    pub capabilities: Option<Capabilities>,
    #[serde(alias = "Metrics", alias = "METRICS")]
    pub metrics: Option<Metrics>,
    #[serde(alias = "Evaluations", alias = "EVALUATIONS")]
    pub evaluations: Option<Evaluations>,
    #[serde(alias = "Tlos", alias = "TLOS")]
    pub tlos: Option<TrainingLearningObjectives>,
    #[serde(alias = "Entities", alias = "ENTITIES")]
    pub entities: Option<Entities>,
    #[serde(alias = "Goals", alias = "GOALS")]
    pub goals: Option<Goals>,
    #[serde(alias = "Injects", alias = "INJECTS")]
    pub injects: Option<Injects>,
    #[serde(alias = "Events", alias = "EVENTS")]
    pub events: Option<Events>,
    #[serde(alias = "Scripts", alias = "SCRIPTS")]
    pub scripts: Option<Scripts>,
    #[serde(alias = "Stories", alias = "STORIES")]
    pub stories: Option<Stories>,
}

impl Scenario {
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(&self).map_err(|e| anyhow!("Failed to serialize to yaml: {}", e))
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let mut schema: Self = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow!("Failed to deserialize from yaml: {}", e))?;
        schema.formalize()?;
        Ok(schema)
    }

    pub fn get_node_dependencies(&self) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();
        if let Some(nodes_value) = &self.nodes {
            for (node_name, _) in nodes_value.iter() {
                dependency_builder = dependency_builder.add_element(node_name.to_string(), vec![]);
            }
        }

        self.build_infrastructure_dependencies(dependency_builder)
    }

    pub fn get_feature_dependencies(&self) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();
        if let Some(features_value) = &self.features {
            for (feature_name, _) in features_value.iter() {
                dependency_builder =
                    dependency_builder.add_element(feature_name.to_string(), vec![]);
            }
        }
        self.build_feature_dependencies(dependency_builder)
    }

    pub fn get_a_node_features_dependencies(
        &self,
        node_feature_name: &str,
    ) -> Result<Dependencies> {
        let mut dependency_builder = Dependencies::builder();

        if let Some(features_value) = &self.features {
            for (feature_name, _) in features_value.iter() {
                if feature_name.eq_ignore_ascii_case(node_feature_name) {
                    dependency_builder =
                        dependency_builder.add_element(feature_name.to_string(), vec![]);
                }
            }
        }
        self.build_a_single_features_dependencies(dependency_builder, node_feature_name)
    }

    fn build_infrastructure_dependencies(
        &self,
        mut dependency_builder: depper::DependenciesBuilder,
    ) -> Result<Dependencies, anyhow::Error> {
        if let Some(infrastructure) = &self.infrastructure {
            for (node_name, infra_node) in infrastructure.iter() {
                let mut dependencies: Vec<String> = vec![];
                if let Some(links) = &infra_node.links {
                    let links = links
                        .iter()
                        .map(|link| link.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(links.as_slice());
                }
                if let Some(node_dependencies) = &infra_node.dependencies {
                    let node_dependencies = node_dependencies
                        .iter()
                        .map(|dependency| dependency.to_string())
                        .collect::<Vec<String>>();
                    dependencies.extend_from_slice(node_dependencies.as_slice());
                }
                dependency_builder =
                    dependency_builder.add_element(node_name.clone(), dependencies);
            }
        }
        dependency_builder.build()
    }

    fn build_feature_dependencies(
        &self,
        mut dependency_builder: depper::DependenciesBuilder,
    ) -> Result<Dependencies, anyhow::Error> {
        if let Some(features) = &self.features {
            for (feature_name, feature) in features.iter() {
                let mut dependencies: Vec<String> = vec![];

                if let Some(feature_dependencies) = &feature.dependencies {
                    dependencies.extend_from_slice(feature_dependencies.as_slice());
                }
                dependency_builder =
                    dependency_builder.add_element(feature_name.to_owned(), dependencies);
            }
        }
        dependency_builder.build()
    }

    fn build_a_single_features_dependencies(
        &self,
        mut dependency_builder: DependenciesBuilder,
        feature_name: &str,
    ) -> Result<Dependencies, anyhow::Error> {
        dependency_builder =
            self.get_a_parent_features_dependencies(feature_name, dependency_builder);

        dependency_builder.build()
    }

    pub fn get_a_parent_features_dependencies(
        &self,
        feature_name: &str,
        mut dependency_builder: DependenciesBuilder,
    ) -> DependenciesBuilder {
        if let Some(features) = &self.features {
            if let Some(feature) = features.get(feature_name) {
                if let Some(dependencies) = &feature.dependencies {
                    dependency_builder = dependency_builder
                        .add_element(feature_name.to_owned(), dependencies.to_vec());

                    for feature_name in dependencies.iter() {
                        dependency_builder = self
                            .get_a_parent_features_dependencies(feature_name, dependency_builder)
                    }
                    return dependency_builder;
                } else {
                    return dependency_builder.add_element(feature_name.to_owned(), vec![]);
                }
            }
        }
        dependency_builder
    }

    fn verify_dependencies(&self) -> Result<()> {
        self.get_node_dependencies()?;
        self.get_feature_dependencies()?;
        Ok(())
    }

    fn verify_switch_counts(&self) -> Result<()> {
        if let Some(infrastructure) = &self.infrastructure {
            if let Some(nodes) = &self.nodes {
                for (node_name, infra_node) in infrastructure.iter() {
                    if infra_node.count > 1 {
                        if let Some(node) = nodes.get(node_name) {
                            if node.type_field == NodeType::Switch {
                                return Err(anyhow!(
                                    "Node {} is a switch with a count higher than 1",
                                    node_name
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn verify_nodes(&self) -> Result<()> {
        let feature_names = self
            .features
            .as_ref()
            .map(|feature_map| feature_map.keys().cloned().collect::<Vec<String>>());
        let condition_names = self
            .conditions
            .as_ref()
            .map(|condition_map| condition_map.keys().cloned().collect::<Vec<String>>());
        let vulnerability_names = self
            .vulnerabilities
            .as_ref()
            .map(|vulnerability_map| vulnerability_map.keys().cloned().collect::<Vec<String>>());
        if let Some(nodes) = &self.nodes {
            for named_node in nodes.iter() {
                let name = named_node.0;
                if name.len() > MAX_LONG_NAME {
                    return Err(anyhow!(
                        "{} is too long, maximum node name length is {:?}",
                        name,
                        MAX_LONG_NAME
                    ));
                }
                Connection::<Feature>::validate_connections(&named_node, &feature_names)?;
                Connection::<Vulnerability>::validate_connections(
                    &named_node,
                    &vulnerability_names,
                )?;
                Connection::<Condition>::validate_connections(
                    &(named_node.0, named_node.1, &self.infrastructure),
                    &condition_names,
                )?;
            }
        }
        Ok(())
    }

    pub fn verify_infrastructure(&self) -> Result<()> {
        let node_names = self
            .nodes
            .as_ref()
            .map(|node_map| node_map.keys().cloned().collect::<Vec<String>>());

        if let Some(infrastructure) = &self.infrastructure {
            infrastructure.keys().try_for_each(|infrastructure_name| {
                Connection::<Infrastructure>::validate_connections(
                    &infrastructure_name,
                    &node_names,
                )?;
                Ok(())
            })?;
        }

        Ok(())
    }

    pub fn verify_evaluations(&self) -> Result<()> {
        let metric_names = self
            .metrics
            .as_ref()
            .map(|metric_map| metric_map.keys().cloned().collect::<Vec<String>>());
        if let Some(evaluations) = &self.evaluations {
            for named_evaluation in evaluations.iter() {
                Connection::<Metric>::validate_connections(&named_evaluation, &metric_names)?;
            }
        }
        Ok(())
    }

    fn verify_training_learning_objectives(&self) -> Result<()> {
        let evaluation_names = self
            .evaluations
            .as_ref()
            .map(|evaluation_map| evaluation_map.keys().cloned().collect::<Vec<String>>());
        let capability_names = self
            .capabilities
            .as_ref()
            .map(|capability_map| capability_map.keys().cloned().collect::<Vec<String>>());

        if let Some(training_learning_objectives) = &self.tlos {
            for named_tlo in training_learning_objectives {
                Connection::<Evaluation>::validate_connections(&named_tlo, &evaluation_names)?;
                Connection::<Capability>::validate_connections(&named_tlo, &capability_names)?;
            }
        }
        Ok(())
    }

    fn verify_metrics(&self) -> Result<()> {
        let condition_names = self
            .conditions
            .as_ref()
            .map(|condition_map| condition_map.keys().cloned().collect::<Vec<String>>());
        if let Some(metrics) = &self.metrics {
            for metric in metrics.iter() {
                metric.validate_connections(&condition_names)?;
            }
        }
        Ok(())
    }

    fn verify_features(&self) -> Result<()> {
        let vulnerability_names = self
            .vulnerabilities
            .as_ref()
            .map(|vulnerability_map| vulnerability_map.keys().cloned().collect::<Vec<String>>());
        if let Some(features) = &self.features {
            for named_feature in features.iter() {
                named_feature.validate_connections(&vulnerability_names)?;
            }
        }
        Ok(())
    }

    fn verify_entities(&self) -> Result<()> {
        let vulnerability_names = self
            .vulnerabilities
            .as_ref()
            .map(|vulnerability_map| vulnerability_map.keys().cloned().collect::<Vec<String>>());
        let tlo_names = self
            .tlos
            .as_ref()
            .map(|tlo_map| tlo_map.keys().cloned().collect::<Vec<String>>());
        if let Some(entities) = &self.entities {
            for named_entity in entities.iter() {
                Connection::<TrainingLearningObjective>::validate_connections(
                    &named_entity,
                    &tlo_names,
                )?;
                Connection::<Vulnerability>::validate_connections(
                    &named_entity,
                    &vulnerability_names,
                )?;
            }
        }
        Ok(())
    }

    fn verify_goals(&self) -> Result<()> {
        let tlo_names = self
            .tlos
            .as_ref()
            .map(|tlo_map| tlo_map.keys().cloned().collect::<Vec<String>>());
        if let Some(goals) = &self.goals {
            for goal in goals.iter() {
                goal.validate_connections(&tlo_names)?;
            }
        }
        Ok(())
    }

    fn verify_capabilities(&self) -> Result<()> {
        let condition_names = self
            .conditions
            .as_ref()
            .map(|condition_map| condition_map.keys().cloned().collect::<Vec<String>>());
        let vulnerability_names = self
            .vulnerabilities
            .as_ref()
            .map(|vulnerability_map| vulnerability_map.keys().cloned().collect::<Vec<String>>());
        if let Some(capabilities) = &self.capabilities {
            for named_capability in capabilities.iter() {
                Connection::<Vulnerability>::validate_connections(
                    &named_capability,
                    &vulnerability_names,
                )?;
                Connection::<Condition>::validate_connections(&named_capability, &condition_names)?;
            }
        }
        Ok(())
    }

    fn verify_injects(&self) -> Result<()> {
        let entity_names = self
            .entities
            .as_ref()
            .map(|entity_map| entity_map.keys().cloned().collect::<Vec<String>>());
        let tlo_names = self
            .tlos
            .as_ref()
            .map(|tlo_map| tlo_map.keys().cloned().collect::<Vec<String>>());
        let capability_names = self
            .capabilities
            .as_ref()
            .map(|capability_map| capability_map.keys().cloned().collect::<Vec<String>>());

        if let Some(injects) = &self.injects {
            for named_inject in injects.iter() {
                Connection::<Entity>::validate_connections(&named_inject, &entity_names)?;
                Connection::<TrainingLearningObjective>::validate_connections(
                    &named_inject,
                    &tlo_names,
                )?;
                Connection::<Capability>::validate_connections(&named_inject, &capability_names)?;
            }
        }
        Ok(())
    }

    fn verify_roles(&self) -> Result<()> {
        if let Some(nodes) = &self.nodes {
            let all_entity_names = self
                .entities
                .clone()
                .map(|entities| entities.flatten().keys().cloned().collect::<Vec<String>>());

            for (node_name, node) in nodes {
                Connection::<Entity>::validate_connections(
                    &(node_name, &node.roles),
                    &all_entity_names,
                )?;

                if let Some(node_features) = &node.features {
                    let feature_roles = node_features.values().cloned().collect::<Vec<String>>();
                    Connection::<Node>::validate_connections(
                        &(node_name, &node.roles),
                        &Some(feature_roles),
                    )?;
                }

                if let Some(node_conditions) = &node.conditions {
                    let condition_roles =
                        node_conditions.values().cloned().collect::<Vec<String>>();
                    Connection::<Node>::validate_connections(
                        &(node_name, &node.roles),
                        &Some(condition_roles),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn verify_events(&self) -> Result<()> {
        let condition_names = self
            .conditions
            .as_ref()
            .map(|entity_map| entity_map.keys().cloned().collect::<Vec<String>>());
        let inject_names = self
            .injects
            .as_ref()
            .map(|tlo_map| tlo_map.keys().cloned().collect::<Vec<String>>());

        if let Some(events) = &self.events {
            for named_event in events.iter() {
                Connection::<Condition>::validate_connections(&named_event, &condition_names)?;
                Connection::<Inject>::validate_connections(&named_event, &inject_names)?;
            }
        }
        Ok(())
    }

    fn verify_scripts(&self) -> Result<()> {
        let event_names = self
            .events
            .as_ref()
            .map(|entity_map| entity_map.keys().cloned().collect::<Vec<String>>());

        if let Some(scripts) = &self.scripts {
            for named_script in scripts.iter() {
                Connection::<Event>::validate_connections(&named_script, &event_names)?;
            }
        }
        Ok(())
    }

    fn verify_stories(&self) -> Result<()> {
        let script_names = self
            .scripts
            .as_ref()
            .map(|entity_map| entity_map.keys().cloned().collect::<Vec<String>>());

        if let Some(stories) = &self.stories {
            for named_story in stories.iter() {
                Connection::<Script>::validate_connections(&named_story, &script_names)?;
            }
        }
        Ok(())
    }
}

impl Formalize for Scenario {
    fn formalize(&mut self) -> Result<()> {
        if let Some(infrastructure_helper) = &self.infrastructure_helper {
            self.infrastructure = Some(Infrastructure::from(infrastructure_helper.clone()));
        }

        if let Some(mut nodes) = self.nodes.clone() {
            nodes.iter_mut().try_for_each(move |(_, node)| {
                node.formalize()?;
                Ok(())
            })?;
            self.nodes = Some(nodes);
        }

        if let Some(mut infrastructure) = self.infrastructure.clone() {
            infrastructure
                .iter_mut()
                .try_for_each(move |(_, infra_node)| {
                    infra_node.formalize()?;
                    Ok(())
                })?;
            self.infrastructure = Some(infrastructure);
        }

        if let Some(features) = &mut self.features {
            features.iter_mut().try_for_each(move |(_, feature)| {
                feature.formalize()?;
                Ok(())
            })?;
        }

        if let Some(mut conditions) = self.conditions.clone() {
            conditions.iter_mut().try_for_each(move |(_, condition)| {
                condition.formalize()?;
                Ok(())
            })?;
            self.conditions = Some(conditions);
        }

        if let Some(mut capabilities) = self.capabilities.clone() {
            capabilities
                .iter_mut()
                .try_for_each(move |(_, condition)| {
                    condition.formalize()?;
                    Ok(())
                })?;
            self.capabilities = Some(capabilities);
        }

        if let Some(mut metrics) = self.metrics.clone() {
            metrics.iter_mut().try_for_each(move |(_, metric)| {
                metric.formalize()?;
                Ok(())
            })?;
            self.metrics = Some(metrics);
        }

        if let Some(mut evaluations) = self.evaluations.clone() {
            evaluations
                .iter_mut()
                .try_for_each(move |(_, evaluation)| {
                    evaluation.formalize()?;
                    Ok(())
                })?;
            self.evaluations = Some(evaluations);
        }

        if let Some(mut vulnerabilities) = self.vulnerabilities.clone() {
            vulnerabilities
                .iter_mut()
                .try_for_each(move |(_, vulnerability)| {
                    vulnerability.formalize()?;
                    Ok(())
                })?;
            self.vulnerabilities = Some(vulnerabilities);
        }

        if let Some(mut goals) = self.goals.clone() {
            goals.iter_mut().try_for_each(move |(_, goal)| {
                goal.formalize()?;
                Ok(())
            })?;
            self.goals = Some(goals);
        }

        if let Some(mut injects) = self.injects.clone() {
            injects.iter_mut().try_for_each(move |(_, inject)| {
                inject.formalize()?;
                Ok(())
            })?;
            self.injects = Some(injects);
        }

        if let Some(mut events) = self.events.clone() {
            events.iter_mut().try_for_each(move |(_, event)| {
                event.formalize()?;
                Ok(())
            })?;
            self.events = Some(events);
        }

        if let Some(mut scripts) = self.scripts.clone() {
            scripts.iter_mut().try_for_each(move |(_, script)| {
                script.formalize()?;
                Ok(())
            })?;
            self.scripts = Some(scripts);
        }

        if let Some(mut stories) = self.stories.clone() {
            stories.iter_mut().try_for_each(move |(_, story)| {
                story.formalize()?;
                Ok(())
            })?;
            self.stories = Some(stories);
        }

        self.verify_entities()?;
        self.verify_goals()?;
        self.verify_nodes()?;
        self.verify_infrastructure()?;
        self.verify_evaluations()?;
        self.verify_switch_counts()?;
        self.verify_features()?;
        self.verify_capabilities()?;
        self.verify_dependencies()?;
        self.verify_metrics()?;
        self.verify_training_learning_objectives()?;
        self.verify_roles()?;
        self.verify_injects()?;
        self.verify_events()?;
        self.verify_scripts()?;
        self.verify_stories()?;
        Ok(())
    }
}

pub fn parse_sdl(sdl_string: &str) -> Result<Scenario> {
    Scenario::from_yaml(sdl_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_minimal_sdl() {
        let minimal_sdl = r#"
                name: test-scenario
                start: 2022-01-20T13:00:00Z
                end: 2022-01-20T23:00:00Z
                
        "#;
        let parsed_schema = parse_sdl(minimal_sdl).unwrap();
        insta::assert_yaml_snapshot!(parsed_schema);
    }

    #[test]
    fn includes_nodes() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1

        "#;
        let nodes = parse_sdl(sdl).unwrap().nodes;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(nodes);
        });
    }

    #[test]
    fn sdl_keys_are_valid_in_lowercase_uppercase_capitalized() {
        let sdl = r#"
            name: test-scenario
            Description: some-description
            start: 2022-01-20T13:00:00Z
            End: 2022-01-20T23:00:00Z
            nodes:
                Win10:
                    TYPE: VM
                    Description: win-10-description
                    resources:
                        RAM: 4 gib
                        Cpu: 2
                    SOURCE:
                        name: windows10
                        version: '*'

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn includes_infrastructure() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
            infrastructure:
                win10:
                    count: 10
                    dependencies:
                        - deb10
                deb10: 3

        "#;
        let infrastructure = parse_sdl(sdl).unwrap().infrastructure;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(infrastructure);
        });
    }

    #[test]
    fn includes_features() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                my-cool-service:
                    type: service
                    source: some-service
                my-cool-config:
                    type: configuration
                    source: some-configuration
                    dependencies:
                        - my-cool-service
                my-cool-artifact:
                    type: artifact
                    source:
                        name: dl_library
                        version: 1.2.3
                    dependencies: 
                        - my-cool-service
        "#;
        let features = parse_sdl(sdl).unwrap().features;
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(features);
        });
    }

    #[test]
    #[should_panic(expected = "Node \"win-10\" has Features but none found under Scenario")]
    fn feature_missing_from_scenario() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                    resources:
                        ram: 4 GiB
                        cpu: 2
                    roles:
                        moderator: "name"
                    features:
                        my-cool-service: "moderator"

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Role admin not found under for Node win-10's roles")]
    fn feature_role_missing_from_node() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                    resources:
                        ram: 4 GiB
                        cpu: 2
                    roles:
                        moderator: "name"
                    features:
                        my-cool-service: "admin"
            features:
                my-cool-service:
                    type: service
                    source: some-service

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn includes_conditions_nodes_and_infrastructure() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                    roles:
                        admin: "username"
                    conditions:
                        condition-1: "admin"
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
                    roles:
                        admin: "username"
                        moderator: "name"
                    conditions:
                        condition-2: "moderator"
                        condition-3: "admin"
            infrastructure:
                win10:
                    count: 1
                    dependencies:
                        - deb10
                deb10: 1
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
                condition-2:
                    source: digital-library-package
                condition-3:
                    command: executable/path.sh
                    interval: 30

        "#;
        let conditions = parse_sdl(sdl).unwrap();
        insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(conditions);
        });
    }

    #[test]
    #[should_panic(expected = "Node \"win10\" can not have count bigger than 1, if it has conditions defined")]
    fn condition_vm_count_in_infrastructure_over_1() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                    roles:
                        admin: "username"
                    conditions:
                        condition-1: "admin"
                deb10:
                    type: VM
                    description: deb-10-description
                    source:
                        name: debian10
                        version: '*'
                    resources:
                        ram: 2 gib
                        cpu: 1
            infrastructure:
                win10:
                    count: 3
                    dependencies:
                        - deb10
                deb10: 1
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Node \"win10\" has Conditions but none found under Scenario")]
    fn condition_doesnt_exist() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win10:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
                    roles:
                        admin: "username"
                    conditions:
                        condition-1: "admin"
            infrastructure:
                win10: 1

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "Role admin not found under for Node win-10's roles")]
    fn condition_role_missing_from_node() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                win-10:
                    type: VM
                    source: windows10
                    resources:
                        ram: 4 GiB
                        cpu: 2
                    roles:
                        moderator: "name"
                    conditions:
                        condition-1: "admin"
            conditions:
                condition-1:
                    command: executable/path.sh
                    interval: 30

        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    #[should_panic(expected = "my-really-really-superlong-non-compliant-name is too long, maximum node name length is 35")]
    fn too_long_node_name_is_disallowed() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            nodes:
                my-really-really-superlong-non-compliant-name:
                    type: VM
                    description: win-10-description
                    source: windows10
                    resources:
                        ram: 4 gib
                        cpu: 2
        "#;
        parse_sdl(sdl).unwrap();
    }

    #[test]
    fn parent_features_dependencies_are_built_correctly() {
        let sdl = r#"
            name: test-scenario
            description: some-description
            start: 2022-01-20T13:00:00Z
            end: 2022-01-20T23:00:00Z
            features:
                parent-service:
                    type: Service
                    source: some-service
                    dependencies: 
                        - child-service
                        - child-config
                child-config:
                    type: Configuration
                    source: some-config
                child-service:
                    type: Service
                    source:
                        name: child-service
                        version: 1.0.0
                    dependencies:
                        - grandchild-config
                        - grandchild-artifact
                grandchild-config:
                    type: Configuration
                    source:
                        name: some-config
                        version: 1.0.0                
                grandchild-artifact:
                    type: Artifact
                    source:
                        name: cool-artifact
                        version: 1.0.0
                    dependencies:
                        - grandgrandchild-artifact
                grandgrandchild-artifact:
                    type: Artifact
                    source: some-artifact
                    dependencies:
                        - grandchild-config
                unrelated-artifact:
                    type: Artifact
                    source: some-artifact
                    dependencies:
                        - very-unrelated-artifact
                very-unrelated-artifact:
                    type: Artifact
                    source: some-artifact
        "#;
        let scenario = parse_sdl(sdl).unwrap();
        let dependencies = scenario
            .get_a_node_features_dependencies("parent-service")
            .unwrap();

        insta::assert_debug_snapshot!(dependencies.generate_tranches().unwrap());
    }
}
