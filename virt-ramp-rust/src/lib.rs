mod bindings {
    use crate::Component;

    wit_bindgen::generate!({ generate_all });

    export!(Component);
}

use bindings::exports::wasi::http::incoming_handler::Guest;
use bindings::wasi::http::types::{
    Fields, IncomingRequest, OutgoingBody, OutgoingResponse, ResponseOutparam,
};
use bindings::wasi::logging::logging::{log, Level};
use std::{fs, io::ErrorKind};

struct Component;

impl Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        // Read the path & query from the request
        let path_with_query = request
            .path_with_query()
            .map(String::from)
            .unwrap_or_else(|| "/".into());
        let (path, _) = path_with_query.split_once('?').unwrap_or(("/", ""));

        log(
            Level::Info,
            "[handle]",
            format!("Request for path: {}", path).as_str(),
        );

        // let body = "Hello, World!";
        let body = match get_file(path) {
            Ok(body) => {
                log(
                    Level::Info,
                    "[handle]",
                    format!("Response: {}", body).as_str(),
                );
                body
            }
            Err(err) => {
                log(Level::Error, "[handle]", format!("Error: {}", err).as_str());
                err
            }
        };

        let response = OutgoingResponse::new(Fields::new());
        let response_body = response.body().expect("response body to exist");
        let stream = response_body.write().unwrap();
        ResponseOutparam::set(response_out, Ok(response));

        // Send back HTTP request
        stream.blocking_write_and_flush(body.as_bytes()).unwrap();
        log(
            Level::Info,
            "[handle]",
            format!("Response: {:?}", body).as_str(),
        );
        drop(stream);
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    }
}

fn get_file(path: &str) -> Result<String, String> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) => {
            let msg = format!("ERR: reading metadata {path}\n{:?}", err);
            log(Level::Info, "[get_file]", msg.as_str());
            return Err(msg);
        }
    };

    log(
        Level::Info,
        "[get_file]",
        format!("Metadata: {:?}", meta).as_str(),
    );

    if meta.is_file() {
        let path = match fs::read_link(&path) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(err) => {
                if err.kind() == ErrorKind::InvalidInput {
                    path.to_string()
                } else {
                    return Err(format!("ERR: {:?}", err));
                }
            }
        };

        log(
            Level::Info,
            "[get_file]",
            format!("Reading file: {}", path).as_str(),
        );

        load_file(path.as_str())
    } else if meta.is_dir() {
        // if path is index, return index.html
        if path.eq("/") {
            load_file(format!("{}/index.html", path).as_str())
        } else {
            load_dir(path)
        }
    } else {
        Err("ERR: Not a file or dir".into())
    }
}

fn load_file(path: &str) -> Result<String, String> {
    match fs::read_to_string(&path) {
        Ok(source) => Ok(source),
        Err(err) => Err(format!("ERR: reading file {path}\n{:?}", err)),
    }
}

fn load_dir(path: &str) -> Result<String, String> {
    let dir = match fs::read_dir(&path) {
        Ok(dir) => dir,
        Err(err) => {
            return Err(format!("ERR: reading dir {path}\n{:?}", err));
        }
    };

    let mut files = String::new();

    for file in dir {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                return Err(format!("ERR: reading dir entry\n{:?}", err));
            }
        };
        files.push_str(match file.file_name().to_str() {
            Some(name) => name,
            None => {
                return Err(format!(
                    "ERR: invalid filename string '{:?}'",
                    file.file_name()
                ));
            }
        });
    }

    Ok(files)
}
