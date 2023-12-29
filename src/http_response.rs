use crate::status_code::StatusCode;
use std::collections::HashMap;
// struct to represent HTTP Response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn serialize(&self) -> String {
        let reason_phrase = self.status_code.to_reason_phrase();
        let mut headers_vec: Vec<String> = Vec::new();

        for header in self.headers.iter() {
            let key = header.0;
            let value = header.1;
            headers_vec.insert(0, format!("{}: {}\r\n", key, value));
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
        .replace('\0', "")
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
                if !(100..=599).contains(&status_code) {
                    return Err("failed to parse status code".into());
                }
                StatusCode::from_u32(status_code)?
            }
            Err(_e) => return Err("failed to parse status code".into()),
        };
        let mut headers = Vec::new();

        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            headers.push(line.to_string());
        }
        let mut headers_map: HashMap<String, String> = HashMap::new();
        for header in headers {
            let components: Vec<&str> = header.splitn(2, ':').collect();
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
fn test_serialize() {
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
        let actual = test_case.input.serialize();

        // assert that actual is equal to expected
        assert_eq!(actual, test_case.expected, "{}", test_case._name);
    }
}

// #[test]
// fn test_from_socket() {
//     let socket = TcpStream::
// }
