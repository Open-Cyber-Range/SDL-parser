use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub name: String,
    pub version: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HelperSource {
    Source(Source),
    ShortSource(String),
}

pub fn get_source(mut source_helper: Option<HelperSource>) -> Option<Source> {
    match &mut source_helper {
        Some(HelperSource::Source(source)) => Some(source.to_owned()),

        Some(HelperSource::ShortSource(source)) => Some(Source {
            name: source.to_owned(),
            version: "*".to_string(),
        }),
        None => None,
    }
}
