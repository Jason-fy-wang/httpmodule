use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse{
    status_code: u16,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new(status: u16) -> Self{
        HttpResponse {
            status_code: status,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: "".to_string(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) -> &mut Self {
        self.headers.insert(key, value);
        self
    }

    pub fn set_body(&mut self, body: String)  -> &mut Self{
        self.body = body;
        self
    }

    pub fn to_string(&self) -> String {
        let mut headers = self.get_headers_string();
        if let Some(body) = self.get_body_string() {
            headers.push_str("Content-Length: ");
            headers.push_str(&body.len().to_string());
            headers.push_str("\r\n");
            headers.push_str("\r\n");
            headers.push_str(body);
        }

        let mut response = self.get_response_line();
        response.push_str(&headers);
        response
    }

    fn get_headers_string(&self) -> String {
        let mut headers = self.headers.iter().map(|(k,v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join("\r\n").to_string();
        headers.push_str("\r\n");
        headers
    }

    fn get_body_string(&self) -> Option<&String> {
        if self.body.is_empty() {
            None
        } else {
            Some(&self.body)
        }

    }

    fn get_response_line(&self) -> String {
        match self.status_code {
            200_u16..=299_u16 => format!("{} {} OK\r\n", self.version, self.status_code),
            300_u16..=399_u16 => format!("{} {} Redirect\r\n", self.version, self.status_code),
            404_u16 => format!("{} {} Not Found\r\n", self.version, self.status_code),
            400_u16..=499_u16 => format!("{} {} Client Error\r\n", self.version, self.status_code),
            500_u16..=599_u16 => format!("{} {} Internal Server Error\r\n", self.version, self.status_code),
            _ => format!("{} {} Unknown Status\r\n", self.version, self.status_code),
        }
    }
    
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    pub fn test_response_create(){
        let mut response = HttpResponse::new(200);
        response.add_header("Content-Type".to_string(), "text/plain".to_string())
            .set_body("Hello, World!".to_string());

        assert_eq!(response.status_code, 200);
        assert_eq!(response.version, "HTTP/1.1");
        assert_eq!(response.headers.get("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(response.body, "Hello, World!");
    }
}