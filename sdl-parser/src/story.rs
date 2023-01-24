use crate::helpers::Connection;
use crate::Formalize;
use crate::{constants::default_clock_value, script::Script};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Story {
    #[serde(default = "default_clock_value", alias = "Clock", alias = "CLOCK")]
    pub clock: u64,
    #[serde(alias = "Scripts", alias = "SCRIPTS")]
    pub scripts: Vec<String>,
}

impl Story {
    pub fn new(potential_clock: Option<u64>) -> Self {
        Self {
            clock: match potential_clock {
                Some(clock) => clock,
                None => default_clock_value(),
            },
            ..Default::default()
        }
    }
}

pub type Stories = HashMap<String, Story>;

impl Formalize for Story {
    fn formalize(&mut self) -> Result<()> {
        if self.scripts.is_empty() {
            return Err(anyhow!("Story must have have at least one Script"));
        }

        if self.clock < 1 {
            return Err(anyhow!("Clock value must be at least 1"));
        }

        Ok(())
    }
}

impl Connection<Script> for (&String, &Story) {
    fn validate_connections(&self, potential_script_names: &Option<Vec<String>>) -> Result<()> {
        if potential_script_names.is_none() {
            return Err(anyhow!(
                "Story is defined but no Scripts declared under Scenario"
            ));
        };

        if let Some(script_names) = potential_script_names {
            for story_script_name in &self.1.scripts {
                if !script_names.contains(story_script_name) {
                    return Err(anyhow!(
                        "Script {story_script_name} not found under Scenario"
                    ));
                }
            }
        }

        Ok(())
    }
}
