/// HTTP status code
#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    /// 200 OK
    OK = 200,
    /// 301 Moved Permanently
    MovedPermanently = 301,
    /// 302 Found
    Found = 302,
    /// 404 Not Found
    NotFound = 404,
    /// 400 Bad Request
    InvalidRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 403 Forbidden
    Forbidden = 403,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 406 Not Acceptable
    NotAcceptable = 406,
    /// 500 Internal Server Error
    InternalServerError = 500,
    /// 501 Not Implemented
    NotImplemented = 501,
    /// 502 Bad Gateway
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
