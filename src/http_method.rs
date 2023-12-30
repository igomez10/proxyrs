use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

// enum for methods implements Display trait
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
    Patch,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
                Method::Options => "OPTIONS",
                Method::Head => "HEAD",
                Method::Trace => "TRACE",
                Method::Connect => "CONNECT",
                Method::Patch => "PATCH",
            }
        )
    }
}

impl FromStr for Method {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "OPTIONS" => Ok(Method::Options),
            "HEAD" => Ok(Method::Head),
            "TRACE" => Ok(Method::Trace),
            "CONNECT" => Ok(Method::Connect),
            "PATCH" => Ok(Method::Patch),
            _ => Err("invalid method".into()),
        }
    }
}
