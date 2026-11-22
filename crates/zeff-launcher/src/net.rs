//! Fetching over the network (Engine and Project Specification, section 2.1; release 0.9.0,
//! step 19b): the engine index, the key list and the archives of an engine.
//!
//! What is fetched is trusted for its signature and never for where it came from, so the
//! network is a narrow thing here: one request for one file, answered with its bytes or with
//! why not. The launcher sends its request and nothing else: no cookie, no identifier of the
//! machine or the user, and its own name and version as the only thing it says of itself.

use crate::trust::HOST;
use std::time::Duration;

/// Why a file was not fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// The server has no such file (404).
    NotFound,
    /// The server answered with another status than success.
    Status(u16),
    /// The answer is larger than anything the launcher fetches.
    TooLarge,
    /// No answer, or one that stopped: the reason in the system's words.
    Failed(String),
}

/// One request for one file.
pub trait Fetch {
    /// The bytes at `url`, when they are at most `limit`.
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, FetchError>;
}

/// The most a small file is: the index, a key list, checksums, a signature.
pub const SMALL: u64 = 4 * 1024 * 1024;

/// The most an archive is.
pub const ARCHIVE: u64 = 2 * 1024 * 1024 * 1024;

/// The network itself: HTTP over TLS, with the certificates this system trusts.
pub struct Http {
    agent: ureq::Agent,
}

impl Http {
    /// `proxy` is `[network] proxy` of the user's settings; without it, `HTTPS_PROXY` and its
    /// kin are honoured as they are everywhere.
    pub fn new(proxy: Option<&str>) -> Http {
        // The one provider of cryptography the launcher is built with; a second call finds it
        // installed and changes nothing.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let proxy = proxy
            .and_then(|proxy| ureq::Proxy::new(proxy).ok())
            .or_else(ureq::Proxy::try_from_env);
        let config = ureq::Agent::config_builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .proxy(proxy)
            .user_agent(format!("zeff-launcher/{}", crate::VERSION))
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .build();
        Http {
            agent: config.into(),
        }
    }
}

impl Fetch for Http {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, FetchError> {
        let failed = |error: ureq::Error| match error {
            ureq::Error::StatusCode(404) => FetchError::NotFound,
            ureq::Error::StatusCode(status) => FetchError::Status(status),
            ureq::Error::BodyExceedsLimit(_) => FetchError::TooLarge,
            other => FetchError::Failed(other.to_string()),
        };
        let mut response = self.agent.get(url).call().map_err(failed)?;
        response
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(failed)
    }
}

/// Where the launcher fetches from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The address everything is under, ending in a slash.
    pub base: String,
    /// It is a mirror the user named, not the publisher's host.
    pub mirror: bool,
}

/// What a mirror was written as, when it is not an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAnAddress(pub String);

impl Source {
    /// The publisher's host, or the mirror that stands for it: `ZEFF_MIRROR`, else `[network]
    /// mirror` of the user's settings. A mirror replaces the host and nothing else.
    pub fn of(
        env: &dyn Fn(&str) -> Option<String>,
        config: Option<&str>,
    ) -> Result<Source, NotAnAddress> {
        let mirror = env("ZEFF_MIRROR")
            .filter(|mirror| !mirror.trim().is_empty())
            .or_else(|| config.and_then(|config| crate::text_at(config, "network", "mirror")));
        let Some(mirror) = mirror else {
            return Ok(Source {
                base: HOST.to_string(),
                mirror: false,
            });
        };
        let address = mirror.trim();
        let rest = address
            .strip_prefix("https://")
            .or_else(|| address.strip_prefix("http://"));
        match rest {
            Some(rest) if !rest.is_empty() && !rest.starts_with('/') => Ok(Source {
                base: format!("{}/", address.trim_end_matches('/')),
                mirror: true,
            }),
            _ => Err(NotAnAddress(mirror)),
        }
    }

    /// The address of `path` under the base.
    pub fn at(&self, path: &str) -> String {
        format!("{}{}", self.base, path.trim_start_matches('/'))
    }
}
