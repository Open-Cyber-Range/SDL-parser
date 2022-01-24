use sdl_parser::{parse_sdl as parse_native_sdl, Schema};
use serde::{Serialize, Deserialize};
use serde_json::{to_string, json};
use std::ffi::{CStr, CString};
use std::ptr;
use libc::{c_char};
use anyhow::Result;

#[derive(Serialize, Deserialize)]
enum Status {
    SUCCESS,
    ERROR
}

#[derive(Serialize, Deserialize)]
struct Response {
    status: Status,
    result: Option<Schema>,
    error_message: Option<String>
}

unsafe fn pointer_to_string<'a>(raw_pointer: *const c_char) -> Result<&'a str> {
    let raw = CStr::from_ptr(raw_pointer);
    return Ok(raw.to_str()?);
}

#[no_mangle]
pub unsafe extern "C" fn parse_sdl_generate(sdl_string_pointer: *const c_char) -> *mut c_char {
    if sdl_string_pointer.is_null() {
        return ptr::null_mut();
    }

    if let Ok(sdl_string) = pointer_to_string(sdl_string_pointer) {
        let json_error_response= json!({
            "status": "ERROR",
            "errorMessage": "failed to serialize response to JSON"
        }).to_string();
    
        let response = match parse_native_sdl(sdl_string) {
            Ok(sld_schema) => {
                to_string(&sld_schema).map_or(json_error_response, |result| result)
            },
            Err(err) => {
                let error_response = Response {
                    status: Status::ERROR,
                    result: None,
                    error_message: Some(err.to_string())
                    
                };
                to_string(&error_response).map_or(json_error_response, |result| result)
            }
        };
        if let Ok(c_str_response) = CString::new(response) {
            return c_str_response.into_raw();
        }
        return ptr::null_mut();
    }

    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn parse_sdl_free(sdl_string: *mut c_char) {
    if sdl_string.is_null() {
        return;
    }
    drop(CString::from_raw(sdl_string));
}
