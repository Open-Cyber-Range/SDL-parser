use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Formalize;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Story {
    #[serde(alias = "Clock", alias = "CLOCK")]
    pub clock: u64,
    #[serde(alias = "Scripts", alias = "SCRIPTS")]
    pub scripts: Vec<String>,
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
