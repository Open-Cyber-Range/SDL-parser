use parser::{parse_sdl as parse_native_sdl, Schema};
use serde::{Serialize, Deserialize};
use serde_json::{to_string, json};

#[derive(Serialize, Deserialize)]
enum Status {
    SUCCESS,
    ERROR
}

#[derive(Serialize, Deserialize)]
struct Result {
    status: Status,
    result: Option<Schema>,
    error_message: Option<String>
}

pub fn parse_sdl(sdl_string: &str) -> String {
    let json_error_response= json!({
        "status": "SUCCESS",
        "errorMessage": "faile to serialize response to JSON"
    }).to_string();

    match parse_native_sdl(sdl_string) {
        Ok(sld_schema) => {
            to_string(&sld_schema).map_or(json_error_response, |result| result)
        },
        Err(err) => {
            let error_result = Result {
                status: Status::ERROR,
                result: None,
                error_message: Some(err.to_string())
                
            };
            to_string(&error_result).map_or(json_error_response, |result| result)
        }
    }
}
