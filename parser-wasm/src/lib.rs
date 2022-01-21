use parser::{parse_sdl as parse_native_sdl};

pub fn parse_sdl(sdl_string: &str) -> String {
    parse_native_sdl(sdl_string).unwrap();
    String::from("")
}

