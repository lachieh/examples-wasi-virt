use serde_json::Value;
use std::collections::HashMap;
use wasmcloud_component::{
    http, info,
    wasi::config::store::{get, get_all},
};

struct Component;

http::export!(Component);

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let path = request.uri().path();
        info!("Path: {}", path);

        let config_route = get("config_route")
            .expect("Failed to get config_route")
            .unwrap_or("/.config.json".to_string());

        info!("Config route: {}", config_route);

        let body = if path == config_route.as_str() {
            info!("Getting public config");
            let public_config = get_public_config().expect("Failed to get public config");
            serde_json::to_string(&public_config).expect("Failed to serialize config")
        } else {
            info!("Returning default message");
            "{\"message\":\"Hello from Rust!\"}".to_string()
        };

        Ok(http::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("Failed to build response"))
    }
}

fn get_public_config() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let config = get_all().expect("Failed to get config");

    let public_config: HashMap<String, String> = config
        .into_iter()
        .filter(|(k, _)| k.starts_with("PUBLIC_"))
        .map(|(k, v)| (k.replace("PUBLIC_", ""), v))
        .collect();

    Ok(public_config)
}
