mod http_client;
mod http_request;
mod http_response;
mod status_code;
mod utils;
extern crate dotenv;
use dotenv::dotenv;
use http_request::HttpRequest;
use http_response::HttpResponse;
use status_code::StatusCode;
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::TcpStream;
use utils::write_to_stream;

use crate::http_client::HTTPClient;

fn main() {
    match dotenv().ok() {
        Some(_) => println!("dotenv loaded"),
        None => println!("dotenv not loaded"),
    }
    env_logger::init();
    println!("Starting rust server");

    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "9095".to_string());
    listen(&address, &port);
}

fn health_handler(socket: &mut TcpStream) {
    let response = http_response::HttpResponse {
        status_code: StatusCode::OK,
        headers: HashMap::from([("Content-Length".to_string(), "2".to_string())]),
        body: "OK".to_string(),
    };

    write_to_stream(socket, &response.serialize()).expect("failed to write to socket");
}

// function to listen incoming tcp connections on port
fn listen(address: &str, port: &str) {
    let listener =
        TcpListener::bind(format!("{}:{}", address, port)).expect("Failed to bind to port");
    log::info!("Listening on port {}", port);
    let client = HTTPClient::new(HashMap::new());

    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                log::info!("incoming request from: {:?}", addr);
                let request_string = match utils::read_from_stream(&mut socket) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("failed to read from stream: {:?}", e);
                        let response = HttpResponse {
                            status_code: StatusCode::InvalidRequest,
                            headers: HashMap::new(),
                            body: "Bad Request".to_string(),
                        };
                        write_to_stream(&mut socket, &response.serialize())
                            .expect("failed to write to socket");
                        close_socket(socket);
                        continue;
                    }
                };

                let request = match HttpRequest::from_string(&request_string) {
                    Ok(request) => request,
                    Err(e) => {
                        log::error!("failed to parse request: {:?}", e);
                        let response = HttpResponse {
                            status_code: StatusCode::InvalidRequest,
                            headers: HashMap::new(),
                            body: "Bad Request".to_string(),
                        };

                        write_to_stream(&mut socket, &response.serialize())
                            .expect("failed to write to socket");
                        close_socket(socket);
                        continue;
                    }
                };

                if request.url.as_str().contains("/health")
                    || request.url.as_str().contains("/favicon.ico")
                // || !request.headers.contains_key("Proxy-Connection")
                {
                    log::debug!("ignore request: {:?}", request.clone());
                    health_handler(&mut socket);
                    close_socket(socket);
                    continue;
                }

                let actual_response = match client.execute(request) {
                    Ok(response) => response,
                    Err(e) => {
                        log::error!("failed to execute request: {:?}", e);
                        let response = HttpResponse {
                            status_code: StatusCode::InternalServerError,
                            headers: HashMap::new(),
                            body: "Bad Request".to_string(),
                        };
                        write_to_stream(&mut socket, &response.serialize())
                            .expect("failed to write to socket");
                        close_socket(socket);
                        continue;
                    }
                };

                write_to_stream(&mut socket, &actual_response.serialize())
                    .expect("failed to write to socket");
                close_socket(socket)
            }
            Err(e) => {
                log::error!("couldn't get client: {:?}", e);
            }
        }
    }
}

fn close_socket(socket: TcpStream) {
    let res = socket.shutdown(std::net::Shutdown::Both);
    match res {
        Ok(_num_bytes) => {
            log::debug!("successfully closed socket");
        }
        Err(_e) => {
            log::error!("failed to close socket {:?}", _e);
        }
    }
}

// tests
#[test]
fn test_listen() {
    use std::thread;

    let _handle = thread::spawn(|| listen("localhost", "5656"));

    let client = HTTPClient::new(HashMap::new());

    let request = HttpRequest {
        method: http_request::Method::Get,
        body: "".to_string(),
        url: url::Url::parse("http://localhost:5656/health").unwrap(),
        headers: HashMap::from([("Host".to_string(), "http://google.com".to_string())]),
    };

    let response = client.execute(request);
    assert!(response.is_ok());
    match response {
        Ok(r) => {
            assert_eq!("OK", r.body.trim())
        }
        Err(_err) => {}
    }
}
