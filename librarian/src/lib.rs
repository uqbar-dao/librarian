cargo_component_bindings::generate!();

use bindings::component::uq_process::types::*;
use bindings::{
    get_payload, print_to_terminal, receive, send_and_await_response, send_request, send_requests,
    send_response, Guest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[allow(dead_code)]
mod process_lib;

struct Component;

fn send_http_response(status: u16, headers: HashMap<String, String>, payload_bytes: Vec<u8>) {
    send_response(
        &Response {
            inherit: false,
            ipc: Some(
                serde_json::json!({
                    "status": status,
                    "headers": headers,
                })
                .to_string(),
            ),
            metadata: None,
        },
        Some(&Payload {
            mime: Some("application/octet-stream".to_string()),
            bytes: payload_bytes,
        }),
    )
}

const LIBRARIAN_PAGE: &str = include_str!("index.html");
const LIBRARIAN_JS: &str = include_str!("index.js");
const LIBRARIAN_CSS: &str = include_str!("index.css");
const WORKER_JS: &str = include_str!("worker.js");

impl Guest for Component {
    fn init(our: Address) {
        print_to_terminal(0, "librarian: start");

        let bindings_address = Address {
            node: our.node.clone(),
            process: ProcessId::from_str("http_bindings:http_bindings:uqbar").unwrap(),
        };

        // <address, request, option<context>, option<payload>>
        let http_endpoint_binding_requests: [(Address, Request, Option<Context>, Option<Payload>);
            3] = [
            (
                bindings_address.clone(),
                Request {
                    inherit: false,
                    expects_response: None,
                    ipc: Some(
                        serde_json::json!({
                            "action": "bind-app",
                            "path": "/librarian",
                            "app": "librarian",
                            "authenticated": false,
                        })
                        .to_string(),
                    ),
                    metadata: None,
                },
                None,
                None,
            ),
            (
                bindings_address.clone(),
                Request {
                    inherit: false,
                    expects_response: None,
                    ipc: Some(
                        serde_json::json!({
                            "action": "bind-app",
                            "path": "/librarian/vector",
                            "app": "librarian",
                            "authenticated": false, // TODO
                        })
                        .to_string(),
                    ),
                    metadata: None,
                },
                None,
                None,
            ),
            (
                bindings_address.clone(),
                Request {
                    inherit: false,
                    expects_response: None,
                    ipc: Some(
                        serde_json::json!({
                            "action": "bind-app",
                            "path": "/librarian/worker.js",
                            "app": "librarian",
                            "authenticated": false, // TODO
                        })
                        .to_string(),
                    ),
                    metadata: None,
                },
                None,
                None,
            ),
        ];
        send_requests(&http_endpoint_binding_requests);

        loop {
            let Ok((source, message)) = receive() else {
                print_to_terminal(0, "librarian: got network error");
                continue;
            };
            let Message::Request(request) = message else {
                print_to_terminal(0, "librarian: got unexpected Response");
                continue;
            };

            let Some(json) = request.ipc else {
                print_to_terminal(0, "librarian: got unexpected Request");
                continue;
            };

            let message_json: serde_json::Value = match serde_json::from_str(&json) {
                Ok(v) => v,
                Err(_) => {
                    print_to_terminal(0, "librarian: failed to parse ipc JSON, skipping");
                    continue;
                }
            };

            if source.process.to_string() == "http_bindings:http_bindings:uqbar" {
                print_to_terminal(0, "librarian: got message from http_bindings");

                let path = message_json["path"].as_str().unwrap_or("");
                let mut default_headers = HashMap::new();
                default_headers.insert("Content-Type".to_string(), "text/html".to_string());

                match path {
                    "/librarian" => {
                        send_http_response(
                            200,
                            default_headers.clone(),
                            LIBRARIAN_PAGE
                                .replace("${node}", &our.node)
                                .replace("${process}", &source.process.to_string())
                                .replace("${js}", LIBRARIAN_JS)
                                .replace("${css}", LIBRARIAN_CSS)
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/librarian/worker.js" => {
                        send_http_response(
                            200,
                            {
                                default_headers.insert("Content-Type".to_string(), "application/javascript".to_string());
                                default_headers
                            },
                            WORKER_JS
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/librarian/vector" => {
                        print_to_terminal(0, "librarian: got request for /librarian/vector");

                        let _drews_res = send_and_await_response(
                            &Address {
                                // always send to drew.uq because we are centralized for now
                                node: "drew.uq".to_string(),
                                process: ProcessId::from_str("server:librarian:drew.uq").unwrap(),
                            },
                            &Request {
                                inherit: false,
                                expects_response: Some(15),
                                ipc: Some(json!({}).to_string()),
                                metadata: None,
                            },
                            Some(
                                &Payload {
                                    mime: Some("application/octet-stream".to_string()),
                                    bytes: get_payload().unwrap().bytes,
                                }
                            ),
                        );
                        print_to_terminal(0, "librarian: got drews res");
        
                        send_http_response(
                            200,
                            {
                                let mut headers = HashMap::new();
                                headers.insert("content-type".to_string(), "application/json".to_string());
                                headers
                            },
                            get_payload().unwrap().bytes
                        );
                    }
                    _ => {
                        send_http_response(
                            404,
                            default_headers.clone(),
                            "Not Found".to_string().as_bytes().to_vec(),
                        );
                        continue;
                    }
                }
            } else {
                print_to_terminal(0, "librarian: got message from source we do not handle");
            }
        }
    }
}
