use std::{path::Path, fmt::{self, Display}};

#[derive(Debug)]
pub enum RequestMethod {
    GET,
    POST
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RequestMethod {
    pub fn from<'a>(s: &'a str) -> RequestMethod {
        if s == "POST" {
            return RequestMethod::POST;
        }
        RequestMethod::GET
    }
}

pub struct Request<'a> {
    pub method: RequestMethod,
    pub uri: &'a Path,
    pub http_version: &'a str,
    pub body: Option<String>
}

pub struct Response {
    pub status: u16,
    pub body: String
}
