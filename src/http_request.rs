use crate::{http_method::Method, utils};
use std::{collections::HashMap, io::Read};
use url::Url;
// struct to represent HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: Url,
    // headers i a map of string to vec string
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn from_stream(stream: &mut dyn Read) -> Result<Self, Box<dyn std::error::Error>> {
        utils::read_request(stream)
    }

    pub fn serialize(&self) -> String {
        let mut request_string = format!("{} {} HTTP/1.1\r\n", self.method, self.url.path());
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
    let mut dummy_request =
        "GET / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n"
            .as_bytes();

    let request = HttpRequest::from_stream(&mut dummy_request).unwrap();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.url, Url::parse("localhost:8080/").unwrap());
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
            input: "GET / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n".to_string(),
            expected: Some(HttpRequest {
                method: Method::Get,
                headers: [
                    ("User-Agent".to_string(), "curl/7.64.1".to_string()),
                    ("Accept".to_string(), "*/*".to_string()),
                    ("Host".to_string(), "localhost:8080".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "".to_string(),
                url: url::Url::parse("localhost:8080/").unwrap(),
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
        let mut stream = test_case.input.as_bytes();
        let actual = match HttpRequest::from_stream(&mut stream) {
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
