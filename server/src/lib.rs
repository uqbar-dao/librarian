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

const PINECONE_API_KEY: &str = include_str!("pinecone-api-key.txt");

struct Component;

impl Guest for Component {
    fn init(our: Address) {
        print_to_terminal(0, "librarian: start");

        loop {
            let Ok((source, message)) = receive() else {
                print_to_terminal(0, "librarian: got network error");
                continue;
            };
            let Message::Request(_request) = message else {
                print_to_terminal(0, "librarian: got unexpected Response");
                continue;
            };

            if source.process.to_string() == "librarian:librarian:drew.uq" {
                print_to_terminal(0, "librarian server: got message from client");
                let _res = send_and_await_response(
                    &Address {
                        node: our.clone().node,
                        process: ProcessId::from_str("http_client:sys:uqbar").unwrap()
                    },
                    &Request {
                        inherit: false,
                        metadata: None,
                        expects_response: Some(10),
                        ipc: Some(json!({
                            "method": "POST",
                            "headers": {
                                "Api-Key": PINECONE_API_KEY,
                                "accept": "application/json",
                                "content-type": "application/json"
                            },
                            "uri": "https://article-recommendations-8a4cf60.svc.us-west4-gcp.pinecone.io/query"
                        }).to_string())
                    },
                    Some(&Payload {
                        mime: Some("application/octet-stream".to_string()),
                        bytes: get_payload().unwrap().bytes,
                    })
                );
                print_to_terminal(0, "librarian server: sending response");
                send_response(
                    &Response {
                        inherit: false,
                        ipc: Some(json!({}).to_string()),
                        metadata: None,
                    },
                    Some(&Payload {
                        mime: Some("application/octet-stream".to_string()),
                        bytes: get_payload().unwrap().bytes,
                    })
                );
            } else {
                print_to_terminal(0, "librarian: got message from unknown source");
            }
        }
    }
}
