use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

/// Request identifier carried through the request lifecycle.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    fn from_header(raw: &HeaderValue) -> Option<Self> {
        raw.to_str()
            .ok()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| RequestId(s.to_string()))
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware: ensure every request has a request_id and every response returns it.
///
/// Behaviour:
/// - If inbound `x-request-id` exists, reuse it.
/// - Otherwise generate a UUIDv4.
/// - Always set `x-request-id` on the response.
///
/// TODO: Add propagation into logs/traces once observability is introduced.
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let req_id = req
        .headers()
        .get("x-request-id")
        .and_then(RequestId::from_header)
        .unwrap_or_else(RequestId::new);

    req.extensions_mut().insert(req_id.clone());

    let mut res = next.run(req).await;

    if let Ok(v) = HeaderValue::from_str(&req_id.0) {
        res.headers_mut().insert("x-request-id", v);
    }

    res
}
