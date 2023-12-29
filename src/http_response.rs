use std::{collections::HashMap, io::Read, net::TcpStream};

// struct to represent HTTP Response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
}

// enum for status code
#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    OK = 200,
    MovedPermanently = 301,
    Found = 302,
    NotFound = 404,
    InvalidRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
}

impl StatusCode {
    pub fn from_u32(status_code: u32) -> Result<Self, Box<dyn std::error::Error>> {
        match status_code {
            200 => Ok(StatusCode::OK),
            301 => Ok(StatusCode::MovedPermanently),
            302 => Ok(StatusCode::Found),
            400 => Ok(StatusCode::InvalidRequest),
            404 => Ok(StatusCode::NotFound),
            401 => Ok(StatusCode::Unauthorized),
            403 => Ok(StatusCode::Forbidden),
            405 => Ok(StatusCode::MethodNotAllowed),
            406 => Ok(StatusCode::NotAcceptable),
            500 => Ok(StatusCode::InternalServerError),
            501 => Ok(StatusCode::NotImplemented),
            502 => Ok(StatusCode::BadGateway),
            0..=99 | 600..=u32::MAX => Err("invalid status code".into()),
            _ => Err("unknown status code".into()),
        }
    }

    pub fn to_reason_phrase(&self) -> &str {
        match self {
            StatusCode::OK => "OK",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::InvalidRequest => "Invalid Request",
            StatusCode::NotFound => "Not Found",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            StatusCode::OK => 200,
            StatusCode::MovedPermanently => 301,
            StatusCode::Found => 302,
            StatusCode::InvalidRequest => 400,
            StatusCode::NotFound => 404,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::NotAcceptable => 406,
            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
        }
    }
}

impl HttpResponse {
    pub fn from_socket(stream: &TcpStream) -> Result<Self, Box<dyn std::error::Error>> {
        // first read headers
        // let headers = stream.
        // validate content_length to know how many bytes to read

        return Err("asdasd".into());
    }
    pub fn to_string(&self) -> String {
        let reason_phrase = self.status_code.to_reason_phrase();
        let mut headers_vec: Vec<String> = Vec::new();

        for header in self.headers.clone() {
            let key = header.0;
            let value = header.1;
            // TODO: fix this hack
            // if key != "Content-Length" {
            headers_vec.insert(0, format!("{}: {}\r\n", key, value));
            // }
        }

        // sort lexicographically
        headers_vec.sort();
        let headers = headers_vec.join("");

        log::debug!("[to_string] headers string in response: {:?}", headers_vec);
        format!(
            "HTTP/1.1 {} {}\r\n{}\r\n{}",
            self.status_code.to_u32(),
            reason_phrase,
            headers,
            self.body
        )
        .replace("\0", "")
    }

    pub fn from_string(response: &str) -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("[from_string] parsing response from: \n{}\n", response);
        let lines: Vec<&str> = response.lines().collect();
        let first_line = lines[0];
        let words: Vec<&str> = first_line.split_whitespace().collect();
        let _http_version = match words[0] {
            "HTTP/1.1" => "HTTP/1.1",
            "HTTP/1.0" => "HTTP/1.0",
            _ => return Err(format!("failed to parse http version: {}", words[0]).into()),
        };

        let status_code = match words[1].parse::<u32>() {
            Ok(status_code) => {
                if status_code < 100 || status_code > 599 {
                    return Err("failed to parse status code".into());
                }
                StatusCode::from_u32(status_code)?
            }
            Err(_e) => return Err("failed to parse status code".into()),
        };
        let mut headers = Vec::new();

        for i in 1..lines.len() {
            if lines[i].is_empty() {
                break;
            }
            headers.push(lines[i].to_string());
        }
        let mut headers_map: HashMap<String, String> = HashMap::new();
        for header in headers {
            let components: Vec<&str> = header.splitn(2, ":").collect();
            if components.len() != 2 {
                return Err(format!("invalid headers {}", header).into());
            }
            let key: String = components[0].to_string();
            let value: String = components[1].trim().to_string();
            headers_map.insert(key, value);
        }
        let content_length: usize = headers_map
            .get("Content-Length")
            .expect("missing header")
            .parse()
            .expect("failed to parse");

        let body = response
            .split("\r\n\r\n")
            .last()
            .unwrap()
            .split_at(content_length)
            .0;
        //     Some(body) => body.to_string(),
        //     None => return Err("failed to parse body".into()),
        // };

        Ok(Self {
            status_code,
            headers: headers_map,
            body: body.to_string(),
        })
    }
}

// test for from_string
#[test]
fn test_from_string() {
    // testcase struct
    struct TestCase {
        _name: String,
        input: String,
        expected: Option<HttpResponse>,
        expected_error: bool,
    }

    // array of testcases
    let test_cases = [
        TestCase {
            _name: "simple 200 OK".to_string(),
            input: "HTTP/1.1 200 OK\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 132\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            expected: Some(HttpResponse {
                status_code: StatusCode::OK,
                headers: [
                    ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Content-Length".to_string(), "132".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            }),
            expected_error: false,
        },
        TestCase {
            _name: "simple 404 Not Found".to_string(),
            input: "HTTP/1.1 404 Not Found\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 132\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            expected: Some(HttpResponse {
                status_code: StatusCode::NotFound,
                headers: [
                    ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Content-Length".to_string(), "132".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            }),
            expected_error: false,
        },
        TestCase{
            _name: "301 Moved Permanently".to_string(),
            input: "HTTP/1.1 301 Moved Permanently\r\nLocation: https://www.example.com/\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 159\r\n\r\n<html>\r\n<head><title>301 Moved Permanently</title></head>\r\n<body>\r\n<p>The document has moved <a href=\"https://www.example.com/\">here</a>.</p>\r\n</body>\r\n</html>".to_string(),
            expected: Some(HttpResponse {
                status_code: StatusCode::MovedPermanently,
                headers: [
                    ("Location".to_string(), "https://www.example.com/".to_string()),
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Content-Length".to_string(), "159".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "<html>\r\n<head><title>301 Moved Permanently</title></head>\r\n<body>\r\n<p>The document has moved <a href=\"https://www.example.com/\">here</a>.</p>\r\n</body>\r\n</html>".to_string(),
            }),
            expected_error: false,
        },
        TestCase{
            _name: "simple 405 Method Not Allowed".to_string(),
            input: "HTTP/1.1 405 Method Not Allowed\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 132\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            expected: Some(HttpResponse {
                status_code: StatusCode::MethodNotAllowed,
                headers: [
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Content-Length".to_string(), "132".to_string()),
                    ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            }),
            expected_error: false,
        },
        TestCase{
            _name: "missing status code".to_string(),
            input: "HTTP/1.1 OK\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 159\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            expected: None,
            expected_error: true,
        },
        TestCase{
            _name: "body from file as 'hello world'".to_string(),
            input: "HTTP/1.1 200 OK\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 11\r\n\r\nhello world".to_string(),
            expected: Some(HttpResponse {
                status_code: StatusCode::OK,
                headers: [
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Content-Length".to_string(), "11".to_string()),
                    ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "hello world".to_string(),
            }),
            expected_error: false,
        },
    ];

    // iterate over testcases
    for test_case in test_cases.iter() {
        // call from_string on each testcase
        let actual = match HttpResponse::from_string(&test_case.input) {
            Ok(response) => response,
            Err(_e) => {
                assert!(test_case.expected_error);
                print!("error: {}", test_case._name);
                continue;
            }
        };

        // assert that actual is equal to expected
        assert_eq!(
            actual.status_code,
            test_case.expected.as_ref().unwrap().status_code,
            "{}",
            test_case._name
        );
        assert_eq!(
            actual.headers,
            test_case.expected.as_ref().unwrap().headers,
            "{}",
            test_case._name
        );
        assert_eq!(
            actual.body,
            test_case.expected.as_ref().unwrap().body,
            "{}",
            test_case._name
        );
    }
}

#[test]
fn test_to_string() {
    // testcase struct
    struct TestCase {
        _name: String,
        input: HttpResponse,
        expected: String,
    }

    // array of testcases
    let test_cases = [
        TestCase {
            _name: "simple 200 OK".to_string(),
            input: HttpResponse {
                status_code: StatusCode::OK,
                headers: [
                    ("Content-Length".to_string(), "138".to_string()),
                    ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
                    ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                body: "<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
            },
            expected: "HTTP/1.1 200 OK\r\nContent-Length: 138\r\nContent-Type: text/html; charset=UTF-8\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
        },
        // TestCase {
        //     _name: "simple 404 Not Found".to_string(),
        //     input: HttpResponse {
        //         status_code: StatusCode::NotFound,
        //         headers: [
        //             ("Date".to_string(), "Mon, 23 May 2023 22:38:34 GMT".to_string()),
        //             ("Content-Type".to_string(), "text/html; charset=UTF-8".to_string()),
        //             ("Content-Length".to_string(), "138".to_string()),
        //         ]
        //         .iter()
        //         .cloned()
        //         .collect(),
        //         body: "<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
        //     },
        //     expected: "HTTP/1.1 404 Not Found\r\nDate: Mon, 23 May".to_string(),
        // },
    ];

    // iterate over testcases
    for test_case in test_cases.iter() {
        // call to_string on each testcase
        let actual = test_case.input.to_string();

        // assert that actual is equal to expected
        assert_eq!(actual, test_case.expected, "{}", test_case._name);
    }
}
