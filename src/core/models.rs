use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub query: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<RequestBody>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestBody {
    Text(String),
    Json(serde_json::Value),
}

impl RequestBody {
    pub fn to_string(&self) -> String {
        match self {
            RequestBody::Text(s) => s.clone(),
            RequestBody::Json(v) => serde_json::to_string(v).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: Uuid,
    pub request_id: Option<Uuid>,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub timing: ResponseTiming,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseTiming {
    pub total_ms: u64,
    pub dns_lookup_ms: Option<u64>,
    pub tcp_connect_ms: Option<u64>,
    pub tls_handshake_ms: Option<u64>,
    pub request_ms: Option<u64>,
}

pub type Collection = Vec<Request>;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    pub request: Request,
    pub timestamp: DateTime<Utc>,
}
