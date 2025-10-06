use oauth2::{HttpRequest, HttpResponse};
use oauth2::http::{Response as RawResponse, Error as HttpError};

/// Concrete error type for the OAuth HTTP adapter.
/// We need a sized, concrete error type so oauth2's generic bounds are satisfied.
#[derive(Debug)]
pub enum OAuthRequestError {
    Reqwest(reqwest::Error),
    Http(HttpError),
}

impl std::fmt::Display for OAuthRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthRequestError::Reqwest(e) => write!(f, "reqwest error: {}", e),
            OAuthRequestError::Http(e) => write!(f, "http error: {}", e),
        }
    }
}

impl std::error::Error for OAuthRequestError {}

/// Small adapter that sends oauth2 HttpRequest using reqwest and returns oauth2 HttpResponse.
/// Returns a concrete error type so it satisfies oauth2's request_async expected bounds.
pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, OAuthRequestError> {
    let client = reqwest::Client::new();

    // Build reqwest request using the http::Request accessors
    let method = request.method().clone();
    let uri = request.uri().to_string();
    let mut req_builder = client.request(method, uri);

    // body() returns &Vec<u8>
    let body = request.body().clone();
    if !body.is_empty() {
        req_builder = req_builder.body(body);
    }

    for (name, value) in request.headers().iter() {
        req_builder = req_builder.header(name.as_str(), value.to_str().unwrap_or_default());
    }

    let resp = req_builder.send().await.map_err(OAuthRequestError::Reqwest)?;
    let status = resp.status().as_u16();
    let bytes = resp.bytes().await.map_err(OAuthRequestError::Reqwest)?;

    // Build an http::Response<Vec<u8>>
    let http_resp = RawResponse::builder()
        .status(status)
        .body(bytes.to_vec())
        .map_err(OAuthRequestError::Http)?;

    Ok(http_resp)
}
