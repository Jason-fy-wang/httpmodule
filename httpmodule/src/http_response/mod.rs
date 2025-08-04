use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse{
    status_code: u16,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new(status: u16, version: &str) -> Self{
        HttpResponse {
            status_code: status,
            version: version.to_string(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

}