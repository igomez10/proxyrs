use crate::http_response::HttpResponse;
use crate::utils::{self, nslookup, write_to_stream};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
};
use serde::{Deserialize, Serialize};


// struct to represent HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: url::Url,
    // headers i a map of string to vec string
    pub headers: HashMap<String, String>,
    pub body: String,
}

// enum for methods implements Display trait
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpRequest {
    pub fn from_string(request: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut lines = request.lines();
        let first_line = lines.next().unwrap();
        let mut words = first_line.split_whitespace();
        
        let method = match words.next(){
            Some(method) => {
                match method {
                    "GET" => Method::GET,
                    "POST" => Method::POST,
                    "PUT" => Method::PUT,
                    "DELETE" => Method::DELETE,
                    _ => return Err("[from_string] failed to parse method".into()),
                }
            },
            None => return Err("[from_string] failed to parse method".into()),
        };

        let url_path = words.next().unwrap();
        
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();
        let mut header_break: bool = false;
        for line in lines.clone() {
            if line.is_empty() {
                header_break = true;
            }
            if !header_break {
                let mut components = line.splitn(2, ":");
                let key: String = components.next().unwrap().to_string();
                let value: String = components.next().unwrap().trim().to_string();
                headers.insert(key, value);
            } else {
                log::debug!("[from_string] line: {}", line);
                body.push_str(line);
            }
        }

        let url = url::Url::parse(url_path).expect("failed to parse url");

        Ok(Self {
            method,
            url,
            headers,
            body,
        })
    }
}


pub struct HTTPClient {
    pub default_headers: HashMap<String, String>,
}

impl HTTPClient {
    pub fn new(default_headers: HashMap<String, String>) -> Self {
        Self {
            default_headers: default_headers
        }
    }

    pub fn execute(&self,request: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let ip_address = nslookup(request.url.host().unwrap().to_string().as_str());
        let port = request.url.port().unwrap_or(80);

        let socket_address = SocketAddr::new(ip_address, port);
        let mut stream = TcpStream::connect(socket_address).expect("failed to connect to server");

        let request_string = request.to_string();
        log::debug!("request string: {}", request_string);
        write_to_stream(&mut stream, &request_string);
        write_to_stream(&mut stream, "\r\n");
        
        // read from socket
        let response_string = utils::read_from_stream(&mut stream);
        let response = HttpResponse::from_string(&response_string).expect("failed to parse response");
        Ok(response)
    }
}

impl HttpRequest {
    pub fn to_string(&self) -> String {
        let mut request_string = format!("{:?} {} HTTP/1.1\r\n", self.method, self.url.path());
        for (key, value) in self.headers.iter() {
            request_string.push_str(format!("{}: {}\r\n", key, value).as_str());
        }
        request_string.push_str("\r\n");
        request_string.push_str(self.body.as_str());
        request_string.push_str("\r\n");
        request_string
    }
}

// test for httpRequest parsing
#[test]
fn test_httprequest_from_string() {
    let dummy_request =
        "GET http://localhost:8080/ HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";

    let request = HttpRequest::from_string(dummy_request).unwrap();
    assert_eq!(request.method, Method::GET);
    assert_eq!(request.url, url::Url::parse("http://localhost:8080/").unwrap());
    assert_eq!(request.headers.get("Host").unwrap(), "localhost:8080");
    assert_eq!(request.headers.get("User-Agent").unwrap(), "curl/7.64.1");
    assert_eq!(request.headers.get("Accept").unwrap(), "*/*");
}

// test for from_string
#[test]
fn test_from_string() {
    // testcase struct
    struct TestCase {
        _name: String,
        input: String,
        expected: Option<HttpRequest>,
        expected_error: bool,
    }

    // array of testcases
    let test_cases = [
        TestCase {
            _name: "simple request".to_string(),
            input: "GET http://localhost:8080 HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n".to_string(),
            expected: Some(HttpRequest {
                method: Method::GET,
                headers: [
                    ("User-Agent".to_string(), "curl/7.64.1".to_string()),
                    ("Accept".to_string(), "*/*".to_string()),
                    ("Host".to_string(), "localhost:8080".to_string()),
                    
                ]
                .iter()
                .cloned()
                .collect(),
                body: "".to_string(),
                url: url::Url::parse("http://localhost:8080/").unwrap(),
            }),
            expected_error: false,
        },
        TestCase{
            _name: "invalid request".to_string(),
            input: "invalid request".to_string(),
            expected: None,
            expected_error: true,
        },

    ];

    // iterate over testcases
    for test_case in test_cases.iter() {
        // call from_string on each testcase
        let actual = match HttpRequest::from_string(&test_case.input) {
            Ok(response) => response,
            Err(_e) => {
                assert!(test_case.expected_error);
                continue;
            }
        };

        // assert that actual is equal to expected
        
        assert_eq!(actual.method, test_case.expected.as_ref().unwrap().method);
        assert_eq!(actual.url, test_case.expected.as_ref().unwrap().url);        
        assert_eq!(actual.headers, test_case.expected.as_ref().unwrap().headers);        
        assert_eq!(actual.body, test_case.expected.as_ref().unwrap().body);
    }
}
