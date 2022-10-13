use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{helpers::Connection, vulnerability::Vulnerability};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Capability {
    #[serde(default, alias = "Description", alias = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(default, alias = "Condition", alias = "CONDITION")]
    pub condition: String,
    #[serde(default, alias = "Vulnerabilities", alias = "VULNERABILITIES")]
    pub vulnerabilities: Vec<String>,
}

pub type Capabilities = HashMap<String, Capability>;

impl Connection<Vulnerability> for (&String, &Capability) {
    fn validate_connections(
        &self,
        potential_vulnerability_names: &Option<Vec<String>>,
    ) -> Result<()> {
        let capability_vulnerabilities = &self.1.vulnerabilities;

        if let Some(vulnerabilities) = potential_vulnerability_names {
            for capability_vulnerability in capability_vulnerabilities.iter() {
                if !vulnerabilities.contains(capability_vulnerability) {
                    return Err(anyhow!(
                        "Vulnerability {} not found under scenario",
                        capability_vulnerability
                    ));
                }
            }
        } else if !capability_vulnerabilities.is_empty() {
            return Err(anyhow!(
                "Vulnerability list is empty under scenario, but capability {} has vulnerabilities",
                self.0
            ));
        }

        Ok(())
    }
}
