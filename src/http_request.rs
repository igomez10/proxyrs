use crate::{http_method::Method, utils};
use std::{collections::HashMap, io::Read};
use url::Url;
// struct to represent HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: url::Url,
    // headers i a map of string to vec string
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn from_stream(stream: &mut dyn Read) -> Result<Self, Box<dyn std::error::Error>> {
        utils::read_request(stream)
    }

    pub fn from_string(request: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = request.lines().collect();
        let first_line = lines[0];
        let words_first_line: Vec<&str> = first_line.split_whitespace().collect();

        let method = match words_first_line[0] {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            _ => return Err("[from_string] failed to parse method".into()),
        };

        // resource can be a path or a url. It will be a url when the request is proxied
        let resource = words_first_line[1];
        let _protocol = words_first_line[2];

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();
        let mut header_break: bool = false;
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                header_break = true;
            }
            if !header_break {
                let components: Vec<&str> = line.splitn(2, ':').collect();
                let key: String = components[0].to_string();
                let value: String = components[1].trim().to_string();
                headers.insert(key, value);
            } else {
                log::debug!("[from_string] line: {}", line);
                body.push_str(line);
            }
        }

        let host_header = headers.get("Host").expect("missing host header");
        let is_proxy_request = resource.starts_with("http://") || resource.starts_with("https://");
        let request_url = if is_proxy_request {
            Url::parse(resource).unwrap()
        } else {
            let url_str = format!("{}{}", host_header, resource);
            Url::parse(&url_str).unwrap()
        };

        Ok(Self {
            method,
            url: request_url,
            headers,
            body,
        })
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
    let dummy_request =
        "GET / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";

    let request = HttpRequest::from_string(dummy_request).unwrap();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.url, url::Url::parse("localhost:8080/").unwrap());
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
