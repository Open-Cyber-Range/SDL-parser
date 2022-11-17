use lazy_static::lazy_static;
use regex::Regex;

pub const DEFAULT_NODE_COUNT: u32 = 1;
pub const fn default_node_count() -> u32 {
    DEFAULT_NODE_COUNT
}

pub const MAX_LONG_NAME: usize = 35;

lazy_static! {
    pub static ref CWE_REGEX: Regex = Regex::new(r"CWE-\d+").unwrap();
}
