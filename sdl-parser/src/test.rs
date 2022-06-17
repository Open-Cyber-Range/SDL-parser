use crate::{parse_sdl, Schema};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEST_SCHEMA: Schema = parse_sdl(TEST_SCHEMA_STRING).unwrap();
}

pub const TEST_SCHEMA_STRING: &str = r#"
scenario:
    name: test-scenario
    description: some-description
    start: 2022-01-20T13:00:00Z
    end: 2022-01-20T23:00:00Z
    infrastructure:
        win10:
            type: VM
            description: win-10-description
            template: windows10
            flavor:
                ram: 4gb
                cpu: 2
        deb10:
            type: VM
            description: deb-10-description
            template: debian10
            flavor:
                ram: 2gb
                cpu: 1
"#;
