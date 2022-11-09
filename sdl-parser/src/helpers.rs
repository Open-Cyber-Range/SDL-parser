use anyhow::{anyhow, Ok, Result};
use std::collections::HashMap;

pub trait Connection<T> {
    fn validate_connections(&self, potential_connections: &Option<Vec<String>>) -> Result<()>;
}

pub fn verify_roles_in_node(
    node_roles: &HashMap<String, String>,
    role_name: &String,
    node_name: &String,
) -> Result<()> {
    if !node_roles.contains_key(role_name) {
        return Err(anyhow!(
            "Role {} not found in node {}",
            role_name,
            node_name
        ));
    }
    Ok(())
}
