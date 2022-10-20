use crate::{parse_sdl, Schema};
use lazy_static::lazy_static;

pub const TEST_SCHEMA_STRING: &str = r#"
scenario:
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

lazy_static! {
    pub static ref TEST_SCHEMA: Schema = parse_sdl(TEST_SCHEMA_STRING).unwrap();
}
