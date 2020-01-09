use hyper::header::{Iter, HeaderName, HeaderMap, HeaderValue};
use std::str::FromStr;
use crate::types::mime_types::MimeType;
use crate::types::error;
use hyper::{StatusCode, Method};

pub enum Content {
    Cache,
    Content(String),
    File(String),
}

pub struct RouteInfo {
    pub url: String,
    pub method: Method,
    pub status_code: StatusCode,
    pub mime_type: MimeType,
    pub headers: HeaderMap,
    pub body: Content
}

impl RouteInfo{
    fn new(url: String, method: String, status_code: u16) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let method = Method::from_str(method.as_str());
        let method = match method {
            Ok(method) => method,
            Err(_e) => {
                println!("invalid method");
                return Err(Box::new(error::Error::new(error::ErrorKind::UnsupportedMethod)));
            }
        };
        let status_code = StatusCode::from_u16(status_code)?;
        Ok(RouteInfo {
            url,
            method,
            status_code,
            mime_type: MimeType::ApplicationOctetStream,
            headers: HeaderMap::new(),
            body: Content::Cache,
        })
    }

    #[allow(dead_code)]
    fn with_default(url: String) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        RouteInfo::new(url, Method::GET.as_str().to_string(), StatusCode::OK.as_u16())
    }

    #[allow(dead_code)]
    fn add_header(&mut self, key: String, value: String) -> bool {
        let header_name = match HeaderName::from_str(key.as_str()) {
            Ok(name) => name,
            Err(e) => {
                println!("error header value: {}", e);
                return false;
            }
        };

        let header_value = match HeaderValue::from_str(value.as_str()) {
            Ok(value) => value,
            Err(e) => {
                println!("error header value: {:?}", e);
                return false;
            }
        };

        self.headers.insert(header_name, header_value);

        true
    }

    #[allow(dead_code)]
    fn remove_header(&mut self, key: String) -> Option<HeaderValue> {
        self.headers.remove(key)
    }

    #[allow(dead_code)]
    fn headers_iter(&mut self) -> Iter<HeaderValue>{
        self.headers.iter()
    }
}