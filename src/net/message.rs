use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use tokio::sync::oneshot::Sender;

use crate::ArcStr;

/// Represents HTTP methods supported by the networking actor.
///
/// This enum provides type-safe HTTP method representation,
/// ensuring that only valid HTTP methods can be used in requests.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    /// HTTP GET method
    Get,
    /// HTTP POST method
    Post,
    /// HTTP PUT method
    Put,
    /// HTTP DELETE method
    Delete,
    /// HTTP PATCH method
    Patch,
}

impl HttpMethod {
    /// Converts the HTTP method to its string representation.
    ///
    /// # Returns
    /// The HTTP method as a string slice.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(anyhow::anyhow!("Invalid HTTP method: {}", s)),
        }
    }
}

/// A key for identifying mocked HTTP requests.
///
/// This struct combines an HTTP method and URL to create a unique identifier
/// for mocked responses, allowing precise control over which requests
/// return which responses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MockRequestKey {
    /// The HTTP method of the request
    pub method: HttpMethod,
    /// The URL of the request
    pub url: ArcStr,
}

impl MockRequestKey {
    /// Creates a new mock request key.
    ///
    /// # Arguments
    /// * `method` - The HTTP method
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key.
    pub fn new(method: HttpMethod, url: ArcStr) -> Self {
        Self { method, url }
    }

    /// Creates a GET request key.
    ///
    /// # Arguments
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key for a GET request.
    pub fn get(url: ArcStr) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    /// Creates a POST request key.
    ///
    /// # Arguments
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key for a POST request.
    pub fn post(url: ArcStr) -> Self {
        Self::new(HttpMethod::Post, url)
    }

    /// Creates a PUT request key.
    ///
    /// # Arguments
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key for a PUT request.
    pub fn put(url: ArcStr) -> Self {
        Self::new(HttpMethod::Put, url)
    }

    /// Creates a DELETE request key.
    ///
    /// # Arguments
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key for a DELETE request.
    pub fn delete(url: ArcStr) -> Self {
        Self::new(HttpMethod::Delete, url)
    }

    /// Creates a PATCH request key.
    ///
    /// # Arguments
    /// * `url` - The URL of the request
    ///
    /// # Returns
    /// A new mock request key for a PATCH request.
    pub fn patch(url: ArcStr) -> Self {
        Self::new(HttpMethod::Patch, url)
    }
}

/// Messages that can be sent to a [`NetCore`] actor.
///
/// This enum defines the different types of network operations that can be performed
/// through the networking actor system.
#[derive(Debug)]
pub enum Message {
    /// Performs an HTTP GET request to the specified URL
    Get {
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Performs an HTTP POST request to the specified URL
    Post {
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Performs an HTTP PUT request to the specified URL
    Put {
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Performs an HTTP DELETE request to the specified URL
    Delete {
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Performs an HTTP PATCH request to the specified URL
    Patch {
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
}
