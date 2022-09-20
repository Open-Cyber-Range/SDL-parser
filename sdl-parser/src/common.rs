use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub name: String,
    pub version: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SourceArray {
    Source(Source),
    ShortSource(String),
}

pub fn get_source(mut source_helper: Option<SourceArray>) -> Option<Source> {
    match &mut source_helper {
        Some(SourceArray::Source(source)) => Some(source.to_owned()),

        Some(SourceArray::ShortSource(source)) => Some(Source {
            name: source.to_owned(),
            version: "*".to_string(),
        }),
        None => None,
    }
}
