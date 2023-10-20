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

const PRELOADED_QUERY: &str = include_str!("query.json");

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

// const librarian_PAGE: &str = include_str!("librarian.html");
// const librarian_JS: &str = include_str!("index.js");
// const librarian_CSS: &str = include_str!("index.css");

impl Guest for Component {
    fn init(our: Address) {
        print_to_terminal(0, "librarian: start");

        let bindings_address = Address {
            node: our.node.clone(),
            process: ProcessId::from_str("http_bindings:http_bindings:uqbar").unwrap(),
        };

        // <address, request, option<context>, option<payload>>
        let http_endpoint_binding_requests: [(Address, Request, Option<Context>, Option<Payload>);
            2] = [
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
                            "authenticated": true,
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

            print_to_terminal(1, format!("librarian: JSON {}", json).as_str());
            let message_json: serde_json::Value = match serde_json::from_str(&json) {
                Ok(v) => v,
                Err(_) => {
                    print_to_terminal(0, "librarian: failed to parse ipc JSON, skipping");
                    continue;
                }
            };

            print_to_terminal(0, "librarian: parsed ipc JSON");

            if source.process.to_string() == "librarian:librarian:uqbar" {
                print_to_terminal(0, "librarian: got message from librarian");
            } else if source.process.to_string() == "http_bindings:http_bindings:uqbar" {
                print_to_terminal(0, "librarian: got message from http_bindings");

                // print_to_terminal(0, &json!(PRELOADED_QUERY).to_string());

                // let vector = json!(PRELOADED_QUERY);

                // print_to_terminal(0, &vector.to_string());


                // let query: Vec<u8> = json!({
                //     "namespace": "default",
                //     "includeValues": true,
                //     "includeMetadata": true,
                //     "topK": 10,
                //     "vector": PRELOADED_QUERY.get(1..PRELOADED_QUERY.len() - 1).unwrap(),
                // }).to_string().as_bytes().to_vec();

                let res = send_and_await_response(
                    &Address {
                        node: our.clone().node,
                        process: ProcessId::from_str("http_client:sys:uqbar").unwrap()
                    },
                    &Request {
                        inherit: false,
                        metadata: None,
                        expects_response: Some(5),
                        ipc: Some(json!({
                            "method": "POST",
                            "headers": {
                                "Api-Key": "",
                                "accept": "application/json",
                                "content-type": "application/json"
                            },
                            "uri": "https://article-recommendations-8a4cf60.svc.us-west4-gcp.pinecone.io/query"
                        }).to_string())
                    },
                    Some(&Payload {
                        mime: Some("application/octet-stream".to_string()),
                        bytes: PRELOADED_QUERY.to_string().as_bytes().to_vec(),
                    })
                );

                send_http_response(
                    200,
                    {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "application/json".to_string());
                        headers
                    },
                    get_payload().unwrap().bytes

                );
            } else {
                print_to_terminal(0, "librarian: got message from unknown source");
            }
        }
    }
}
