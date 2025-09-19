use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ArcStr;
use crate::net::message::MockRequestKey;

/// Mock implementation of the Net actor for testing purposes.
///
/// This struct contains predefined HTTP responses for various request types,
/// allowing tests to run without making actual network requests.
#[derive(Debug, Clone)]
pub struct Mock {
    responses: Arc<Mutex<HashMap<MockRequestKey, ArcStr>>>,
}

impl Mock {
    /// Creates a new mock instance with the provided responses.
    ///
    /// # Arguments
    /// * `responses` - Initial response cache mapping HTTP method + URL pairs to responses
    pub fn new(responses: HashMap<MockRequestKey, ArcStr>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }

    /// Creates a new mock instance with an empty response cache.
    pub fn empty() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Performs an HTTP GET request using mock responses.
    ///
    /// # Arguments
    /// * `url` - The URL to send the GET request to
    /// * `headers` - Optional headers to include in the request (ignored in mock)
    ///
    /// # Returns
    /// The response body as a string, or an error if not found in mock responses.
    pub async fn get(
        &self,
        url: ArcStr,
        _headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> Result<ArcStr, anyhow::Error> {
        let responses = self.responses.lock().await;
        let key = MockRequestKey::get(url);
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("GET request not found in mock responses: {}", key.url)
        })
    }

    /// Performs an HTTP POST request using mock responses.
    ///
    /// # Arguments
    /// * `url` - The URL to send the POST request to
    /// * `headers` - Optional headers to include in the request (ignored in mock)
    /// * `body` - Optional body content to send with the request (ignored in mock)
    ///
    /// # Returns
    /// The response body as a string, or an error if not found in mock responses.
    pub async fn post(
        &self,
        url: ArcStr,
        _headers: Option<HashMap<ArcStr, ArcStr>>,
        _body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        let responses = self.responses.lock().await;
        let key = MockRequestKey::post(url);
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("POST request not found in mock responses: {}", key.url)
        })
    }

    /// Performs an HTTP PUT request using mock responses.
    ///
    /// # Arguments
    /// * `url` - The URL to send the PUT request to
    /// * `headers` - Optional headers to include in the request (ignored in mock)
    /// * `body` - Optional body content to send with the request (ignored in mock)
    ///
    /// # Returns
    /// The response body as a string, or an error if not found in mock responses.
    pub async fn put(
        &self,
        url: ArcStr,
        _headers: Option<HashMap<ArcStr, ArcStr>>,
        _body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        let responses = self.responses.lock().await;
        let key = MockRequestKey::put(url);
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("PUT request not found in mock responses: {}", key.url)
        })
    }

    /// Performs an HTTP DELETE request using mock responses.
    ///
    /// # Arguments
    /// * `url` - The URL to send the DELETE request to
    /// * `headers` - Optional headers to include in the request (ignored in mock)
    ///
    /// # Returns
    /// The response body as a string, or an error if not found in mock responses.
    pub async fn delete(
        &self,
        url: ArcStr,
        _headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> Result<ArcStr, anyhow::Error> {
        let responses = self.responses.lock().await;
        let key = MockRequestKey::delete(url);
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("DELETE request not found in mock responses: {}", key.url)
        })
    }

    /// Performs an HTTP PATCH request using mock responses.
    ///
    /// # Arguments
    /// * `url` - The URL to send the PATCH request to
    /// * `headers` - Optional headers to include in the request (ignored in mock)
    /// * `body` - Optional body content to send with the request (ignored in mock)
    ///
    /// # Returns
    /// The response body as a string, or an error if not found in mock responses.
    pub async fn patch(
        &self,
        url: ArcStr,
        _headers: Option<HashMap<ArcStr, ArcStr>>,
        _body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        let responses = self.responses.lock().await;
        let key = MockRequestKey::patch(url);
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("PATCH request not found in mock responses: {}", key.url)
        })
    }
}
