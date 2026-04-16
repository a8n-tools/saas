//! HTTP client for Forgejo's OCI distribution endpoints (/v2/*).
//!
//! Validates that every upstream URL is under the configured `base_url`
//! before forwarding the Forgejo API token, matching the safety rule in
//! `services::forgejo`.

use bytes::Bytes;
use futures_util::Stream;
use reqwest::{Client, Response};
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not found")]
    NotFound,
    #[error("upstream status {0}")]
    Upstream(u16),
    #[error("invalid upstream url")]
    InvalidUrl,
    #[error("url parse: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// A manifest response from upstream: raw bytes + media type + digest.
#[derive(Debug)]
pub struct ManifestResponse {
    pub bytes: Bytes,
    pub media_type: String,
    pub digest: String,
}

/// A streamed blob response: headers + body stream.
pub struct BlobStream {
    pub content_length: Option<u64>,
    pub media_type: Option<String>,
    pub digest: Option<String>,
    pub body: Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin>,
}

#[derive(Clone)]
pub struct ForgejoRegistryClient {
    http: Client,
    base_url: String,
    token: String,
}

impl ForgejoRegistryClient {
    pub fn new(base_url: String, token: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client builds");
        Self { http, base_url, token }
    }

    fn validate_host(&self, url: &Url) -> Result<(), RegistryError> {
        let base = Url::parse(&self.base_url)?;
        if url.scheme() != base.scheme()
            || url.host_str() != base.host_str()
            || url.port_or_known_default() != base.port_or_known_default()
        {
            return Err(RegistryError::InvalidUrl);
        }
        Ok(())
    }

    /// GET /v2/<owner>/<name>/manifests/<reference>
    pub async fn get_manifest(
        &self,
        owner: &str,
        name: &str,
        reference: &str,
        accept: &str,
    ) -> Result<ManifestResponse, RegistryError> {
        let url = format!(
            "{}/v2/{}/{}/manifests/{}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(owner),
            urlencoding::encode(name),
            urlencoding::encode(reference),
        );
        let parsed = Url::parse(&url)?;
        self.validate_host(&parsed)?;

        let resp = self.http.get(parsed)
            .basic_auth("", Some(&self.token))
            .header("Accept", accept)
            .send()
            .await?;

        self.map_status(&resp)?;
        let media_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/vnd.oci.image.manifest.v1+json")
            .to_string();
        let digest = resp
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = resp.bytes().await?;
        Ok(ManifestResponse { bytes, media_type, digest })
    }

    /// GET /v2/<owner>/<name>/blobs/<digest>
    pub async fn get_blob(
        &self,
        owner: &str,
        name: &str,
        digest: &str,
    ) -> Result<BlobStream, RegistryError> {
        let url = format!(
            "{}/v2/{}/{}/blobs/{}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(owner),
            urlencoding::encode(name),
            urlencoding::encode(digest),
        );
        let parsed = Url::parse(&url)?;
        self.validate_host(&parsed)?;

        let resp = self.http.get(parsed)
            .basic_auth("", Some(&self.token))
            .send()
            .await?;

        self.map_status(&resp)?;
        let content_length = resp.content_length();
        let media_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let header_digest = resp
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let stream = resp.bytes_stream();
        Ok(BlobStream {
            content_length,
            media_type,
            digest: header_digest,
            body: Box::new(stream),
        })
    }

    fn map_status(&self, resp: &Response) -> Result<(), RegistryError> {
        match resp.status().as_u16() {
            200 => Ok(()),
            404 => Err(RegistryError::NotFound),
            code if (500..600).contains(&code) => Err(RegistryError::Upstream(code)),
            code => Err(RegistryError::Upstream(code)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[actix_rt::test]
    async fn rejects_upstream_url_outside_base() {
        let client = ForgejoRegistryClient::new(
            "https://git.example.com".into(),
            "tok".into(),
        );
        let bad = Url::parse("https://evil.example.com/v2/a/b/blobs/sha256:x").unwrap();
        assert!(matches!(client.validate_host(&bad), Err(RegistryError::InvalidUrl)));

        let good = Url::parse("https://git.example.com/v2/a/b/blobs/sha256:x").unwrap();
        assert!(client.validate_host(&good).is_ok());
    }

    #[actix_rt::test]
    async fn get_manifest_returns_headers_and_body() {
        let server = MockServer::start().await;
        let body = br#"{"mediaType":"application/vnd.oci.image.manifest.v1+json","layers":[]}"#;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(body.to_vec())
                    .insert_header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                    .insert_header("Docker-Content-Digest", "sha256:abc"),
            )
            .mount(&server)
            .await;

        let client = ForgejoRegistryClient::new(server.uri(), "tok".into());
        let mr = client.get_manifest("a", "b", "v1", "application/vnd.oci.image.manifest.v1+json").await.unwrap();
        assert_eq!(mr.media_type, "application/vnd.oci.image.manifest.v1+json");
        assert_eq!(mr.digest, "sha256:abc");
        assert_eq!(mr.bytes, Bytes::from(&body[..]));
    }

    #[actix_rt::test]
    async fn upstream_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = ForgejoRegistryClient::new(server.uri(), "tok".into());
        let err = client.get_manifest("a", "b", "v1", "application/json").await.unwrap_err();
        assert!(matches!(err, RegistryError::NotFound));
    }
}
