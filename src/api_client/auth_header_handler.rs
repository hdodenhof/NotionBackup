use regex::Regex;
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, SET_COOKIE};
use std::error::Error;

pub struct AuthHeaderHandler {
    header_pattern: Regex,
}

impl AuthHeaderHandler {
    pub fn new() -> AuthHeaderHandler {
        AuthHeaderHandler {
            header_pattern: Regex::new(r"token_v2=(.*); Domain=.*").unwrap(),
        }
    }

    pub fn build_auth_header(&self, token: &str) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        header_map.insert(
            COOKIE,
            HeaderValue::from_str(&self.build_token_header(token)).unwrap()
        );

        header_map
    }

    fn build_token_header(&self, token: &str) -> String {
        format!("token_v2={}", &token)
    }

    pub fn parse_auth_header(&self, headers: &HeaderMap) -> Result<String, Box<dyn Error>> {
        let token_header = self.find_token_header(headers)?;
        let token = self.parse_token(token_header)?;

        Ok(token)
    }

    fn find_token_header<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Box<dyn Error>> {
        let token_headers: Vec<&str> = headers.get_all(SET_COOKIE).iter()
            .map(|v| v.to_str().unwrap_or("<error>"))
            .filter(|&v| self.header_pattern.is_match(v))
            .collect();

        if token_headers.len() != 1 {
            Err("Unexpected number of auth token headers")?;
        }

        let token_header = token_headers.get(0).unwrap();

        Ok(token_header)
    }

    fn parse_token(&self, header: &str) -> Result<String, Box<dyn Error>> {
        let token = match self.header_pattern.captures(header) {
            Some(captures) => {
                match captures.get(1) {
                    Some(value) => value.as_str(),
                    None => Err("Parsing auth token from header failed")?
                }
            },
            None => Err("Parsing auth token from header failed")?
        };

        Ok(String::from(token))
    }
}
