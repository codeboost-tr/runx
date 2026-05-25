// rust-style-allow: large-file because the runtime HTTP transport keeps request
// modeling, header validation, status parsing, and security-focused unit tests
// in one review unit.
use std::fmt;
#[cfg(any(feature = "async-http", test))]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
#[cfg(feature = "async-http")]
use std::time::Duration;

#[cfg(any(feature = "async-http", test))]
use url::Url;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HostedHttpHeader {
    pub name: String,
    pub value: String,
}

impl HostedHttpHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl fmt::Debug for HostedHttpHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostedHttpHeader")
            .field("name", &self.name)
            .field(
                "value",
                &if sensitive_header_name(&self.name) {
                    "[redacted]"
                } else {
                    self.value.as_str()
                },
            )
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HostedHttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<HostedHttpHeader>,
    pub body: Option<String>,
}

impl fmt::Debug for HostedHttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostedHttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field(
                "body",
                &self.body.as_ref().map(|_| "[redacted body present]"),
            )
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HostedHttpResponse {
    pub status: u16,
    pub body: String,
}

impl fmt::Debug for HostedHttpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostedHttpResponse")
            .field("status", &self.status)
            .field("body", &format_args!("{} bytes", self.body.len()))
            .finish()
    }
}

pub trait HostedTransport {
    fn send(&self, request: HostedHttpRequest) -> Result<HostedHttpResponse, HostedHttpError>;
}

#[derive(Clone, Debug)]
pub struct ReqwestHttpTransport {
    #[cfg(feature = "async-http")]
    client: reqwest::Client,
    #[cfg(feature = "async-http")]
    allow_private_networks: bool,
}

#[cfg(feature = "async-http")]
impl ReqwestHttpTransport {
    pub fn new() -> Result<Self, HostedHttpError> {
        Self::with_timeouts(Duration::from_secs(30), Duration::from_secs(10))
    }

    fn with_timeouts(
        request_timeout: Duration,
        connect_timeout: Duration,
    ) -> Result<Self, HostedHttpError> {
        // reqwest is built with `rustls-no-provider`, so the process needs a
        // default crypto provider before a TLS client can be constructed.
        // Install ring once; an Err means another transport already set it.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(request_timeout)
            .connect_timeout(connect_timeout)
            .build()
            .map_err(|error| HostedHttpError::Transport {
                message: error.to_string(),
            })?;
        Ok(Self {
            client,
            allow_private_networks: false,
        })
    }

    #[cfg(test)]
    fn with_private_network_access_for_tests() -> Result<Self, HostedHttpError> {
        let mut transport = Self::with_timeouts(Duration::from_secs(30), Duration::from_secs(10))?;
        transport.allow_private_networks = true;
        Ok(transport)
    }

    #[cfg(test)]
    fn with_private_network_timeouts_for_tests(
        request_timeout: Duration,
        connect_timeout: Duration,
    ) -> Result<Self, HostedHttpError> {
        let mut transport = Self::with_timeouts(request_timeout, connect_timeout)?;
        transport.allow_private_networks = true;
        Ok(transport)
    }
}

#[cfg(feature = "async-http")]
impl HostedTransport for ReqwestHttpTransport {
    fn send(&self, request: HostedHttpRequest) -> Result<HostedHttpResponse, HostedHttpError> {
        validate_http_url(&request.url, self.allow_private_networks)?;
        let client = self.client.clone();
        block_on_http(async move {
            let method = reqwest_method(request.method);
            let mut builder = client.request(method, request.url);
            for header in request.headers {
                validate_header(&header)?;
                let name = reqwest::header::HeaderName::from_bytes(header.name.trim().as_bytes())
                    .map_err(|error| HostedHttpError::InvalidHeaderName {
                    name: header.name.clone(),
                    message: error.to_string(),
                })?;
                let value =
                    reqwest::header::HeaderValue::from_str(&header.value).map_err(|error| {
                        HostedHttpError::InvalidHeaderValue {
                            name: header.name.clone(),
                            message: error.to_string(),
                        }
                    })?;
                builder = builder.header(name, value);
            }
            if let Some(body) = request.body {
                builder = builder.body(body);
            }
            let response = builder
                .send()
                .await
                .map_err(|error| HostedHttpError::Transport {
                    message: error.to_string(),
                })?;
            let status = response.status().as_u16();
            let body = response
                .text()
                .await
                .map_err(|error| HostedHttpError::TransportDecode {
                    message: error.to_string(),
                })?;
            Ok(HostedHttpResponse { status, body })
        })
    }
}

#[derive(Clone, Debug)]
#[cfg(any(feature = "async-http", test))]
#[allow(dead_code)]
pub struct HostedHttpClient<T = ReqwestHttpTransport> {
    base_url: String,
    transport: T,
}

#[cfg(any(feature = "async-http", test))]
#[allow(dead_code)]
impl<T: HostedTransport> HostedHttpClient<T> {
    pub fn with_transport(
        base_url: impl AsRef<str>,
        transport: T,
    ) -> Result<Self, HostedHttpError> {
        let base_url = strip_one_trailing_slash(base_url.as_ref());
        validate_http_url(&base_url, false)?;
        Ok(Self {
            base_url,
            transport,
        })
    }

    pub fn route_url(&self, route: &str) -> Result<String, HostedHttpError> {
        let normalized_route = route.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, normalized_route);
        validate_http_url(&url, false)?;
        Ok(url)
    }

    pub fn request(
        &self,
        method: HttpMethod,
        route: &str,
    ) -> Result<HostedHttpRequest, HostedHttpError> {
        Ok(HostedHttpRequest {
            method,
            url: self.route_url(route)?,
            headers: Vec::new(),
            body: None,
        })
    }

    pub fn send(&self, request: HostedHttpRequest) -> Result<HostedHttpResponse, HostedHttpError> {
        self.transport.send(request)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HostedHttpError {
    #[error("invalid hosted HTTP url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("hosted HTTP transport failed: {message}")]
    Transport { message: String },
    #[error("hosted HTTP transport cannot block inside an active async runtime")]
    BlockingHttpInsideAsyncRuntime,
    #[error("hosted HTTP async runtime is unavailable: {message}")]
    AsyncRuntimeUnavailable { message: String },
    #[error("hosted HTTP transport returned invalid output: {message}")]
    TransportDecode { message: String },
    #[error("unsupported hosted HTTP url scheme '{scheme}': only http and https are allowed")]
    UnsupportedUrlScheme { scheme: String },
    #[error("hosted HTTP url host '{host}' is not publicly routable")]
    PrivateNetworkUrl { host: String },
    #[error("invalid hosted HTTP header name '{name}': {message}")]
    InvalidHeaderName { name: String, message: String },
    #[error("invalid hosted HTTP header value for '{name}': {message}")]
    InvalidHeaderValue { name: String, message: String },
}

#[cfg(any(feature = "async-http", test))]
#[allow(dead_code)]
fn strip_one_trailing_slash(value: &str) -> String {
    value.strip_suffix('/').unwrap_or(value).to_owned()
}

fn sensitive_header_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized == "authorization"
        || normalized == "proxy-authorization"
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("api-key")
}

#[cfg(feature = "async-http")]
fn validate_header(header: &HostedHttpHeader) -> Result<(), HostedHttpError> {
    let name = header.name.trim();
    if name.is_empty() || !name.bytes().all(is_header_token_byte) {
        return Err(HostedHttpError::InvalidHeaderName {
            name: header.name.clone(),
            message: "header names must be HTTP token characters".to_owned(),
        });
    }
    if header.value.contains('\r') || header.value.contains('\n') {
        return Err(HostedHttpError::InvalidHeaderValue {
            name: header.name.clone(),
            message: "header values must not contain line breaks".to_owned(),
        });
    }
    Ok(())
}

#[cfg(any(feature = "async-http", test))]
#[allow(dead_code)]
fn validate_http_url(value: &str, allow_private_networks: bool) -> Result<(), HostedHttpError> {
    let url = Url::parse(value)?;
    match url.scheme() {
        "http" | "https" => validate_public_host(&url, allow_private_networks),
        scheme => Err(HostedHttpError::UnsupportedUrlScheme {
            scheme: scheme.to_owned(),
        }),
    }
}

#[cfg(any(feature = "async-http", test))]
fn validate_public_host(url: &Url, allow_private_networks: bool) -> Result<(), HostedHttpError> {
    if allow_private_networks {
        return Ok(());
    }
    let Some(host) = url.host_str() else {
        return Err(HostedHttpError::PrivateNetworkUrl {
            host: "<missing>".to_owned(),
        });
    };
    let normalized = host.trim_end_matches('.').to_ascii_lowercase();
    if normalized == "localhost"
        || normalized.ends_with(".localhost")
        || normalized == "metadata.google.internal"
    {
        return Err(HostedHttpError::PrivateNetworkUrl {
            host: host.to_owned(),
        });
    }
    let ip_host = normalized
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(&normalized);
    if let Ok(ip) = ip_host.parse::<IpAddr>() {
        if is_private_network_ip(ip) {
            return Err(HostedHttpError::PrivateNetworkUrl {
                host: host.to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(any(feature = "async-http", test))]
fn is_private_network_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_private_network_ipv4(ip),
        IpAddr::V6(ip) => is_private_network_ipv6(ip),
    }
}

#[cfg(any(feature = "async-http", test))]
fn is_private_network_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || ip.is_multicast()
        || ip.octets() == [169, 254, 169, 254]
}

#[cfg(any(feature = "async-http", test))]
fn is_private_network_ipv6(ip: Ipv6Addr) -> bool {
    ip.to_ipv4_mapped().is_some_and(is_private_network_ipv4)
        || ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || is_unique_local_ipv6(ip)
        || is_unicast_link_local_ipv6(ip)
        || is_documentation_ipv6(ip)
}

#[cfg(any(feature = "async-http", test))]
fn is_unique_local_ipv6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

#[cfg(any(feature = "async-http", test))]
fn is_unicast_link_local_ipv6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(any(feature = "async-http", test))]
fn is_documentation_ipv6(ip: Ipv6Addr) -> bool {
    ip.segments()[0] == 0x2001 && ip.segments()[1] == 0x0db8
}

#[cfg(feature = "async-http")]
fn reqwest_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Delete => reqwest::Method::DELETE,
    }
}

#[cfg(feature = "async-http")]
fn block_on_http<F, T>(future: F) -> Result<T, HostedHttpError>
where
    F: std::future::Future<Output = Result<T, HostedHttpError>>,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(HostedHttpError::BlockingHttpInsideAsyncRuntime);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| HostedHttpError::AsyncRuntimeUnavailable {
            message: error.to_string(),
        })?;
    runtime.block_on(future)
}

#[cfg(feature = "async-http")]
fn is_header_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io;
    #[cfg(feature = "async-http")]
    use std::io::{Read, Write};
    #[cfg(feature = "async-http")]
    use std::net::TcpListener;
    #[cfg(feature = "async-http")]
    use std::time::Duration;

    #[cfg(feature = "async-http")]
    use super::ReqwestHttpTransport;
    use super::{
        HostedHttpClient, HostedHttpError, HostedHttpHeader, HostedHttpRequest, HostedHttpResponse,
        HostedTransport, HttpMethod,
    };

    #[derive(Default)]
    struct MockTransport {
        requests: RefCell<Vec<HostedHttpRequest>>,
    }

    impl HostedTransport for &MockTransport {
        fn send(&self, request: HostedHttpRequest) -> Result<HostedHttpResponse, HostedHttpError> {
            self.requests.borrow_mut().push(request);
            Ok(HostedHttpResponse {
                status: 204,
                body: String::new(),
            })
        }
    }

    #[derive(Debug, thiserror::Error)]
    enum HostedHttpTestError {
        #[error(transparent)]
        HostedHttp(#[from] HostedHttpError),
        #[error(transparent)]
        Io(#[from] io::Error),
        #[cfg(feature = "async-http")]
        #[error("server thread panicked")]
        ServerThread,
    }

    #[test]
    fn client_normalizes_base_url_and_routes_requests() -> Result<(), HostedHttpTestError> {
        let transport = MockTransport::default();
        let client = HostedHttpClient::with_transport("https://api.example/", &transport)?;

        let mut request = client.request(HttpMethod::Delete, "/v1/grants/grant_1")?;
        request
            .headers
            .push(HostedHttpHeader::new("accept", "application/json"));
        request.body = Some("{\"ok\":true}".to_owned());
        let response = client.send(request)?;

        assert_eq!(response.status, 204);
        let sent = transport.requests.borrow();
        assert_eq!(sent[0].method, HttpMethod::Delete);
        assert_eq!(sent[0].url, "https://api.example/v1/grants/grant_1");
        assert_eq!(sent[0].headers[0].name, "accept");
        assert_eq!(sent[0].body.as_deref(), Some("{\"ok\":true}"));
        Ok(())
    }

    #[test]
    fn debug_output_redacts_sensitive_header_values() {
        let request = HostedHttpRequest {
            method: HttpMethod::Get,
            url: "https://api.example/v1/grants".to_owned(),
            headers: vec![
                HostedHttpHeader::new("authorization", "Bearer SECRET_CONNECT_TOKEN"),
                HostedHttpHeader::new("x-runx-token", "SECRET_HEADER_TOKEN"),
                HostedHttpHeader::new("accept", "application/json"),
            ],
            body: Some("SECRET_BODY".to_owned()),
        };

        let debug = format!("{request:?}");
        assert!(!debug.contains("SECRET_CONNECT_TOKEN"));
        assert!(!debug.contains("SECRET_HEADER_TOKEN"));
        assert!(!debug.contains("SECRET_BODY"));
        assert!(debug.contains("[redacted]"));
        assert!(debug.contains("application/json"));
    }

    #[test]
    fn invalid_base_urls_fail_closed() {
        assert!(HostedHttpClient::with_transport("not a url", &MockTransport::default()).is_err());
        assert!(matches!(
            HostedHttpClient::with_transport("file:///tmp/runx.sock", &MockTransport::default()),
            Err(HostedHttpError::UnsupportedUrlScheme { .. })
        ));
    }

    #[test]
    fn private_network_base_urls_fail_closed() {
        for value in [
            "http://localhost",
            "http://service.localhost",
            "http://127.0.0.1",
            "http://10.0.0.1",
            "http://172.16.0.1",
            "http://192.168.0.1",
            "http://169.254.169.254",
            "http://[::1]",
            "http://[::ffff:127.0.0.1]",
            "http://[fc00::1]",
            "http://[fe80::1]",
            "http://metadata.google.internal",
        ] {
            assert!(
                matches!(
                    HostedHttpClient::with_transport(value, &MockTransport::default()),
                    Err(HostedHttpError::PrivateNetworkUrl { .. })
                ),
                "{value} should be rejected as private"
            );
        }
    }

    #[test]
    fn public_base_urls_are_allowed() -> Result<(), HostedHttpTestError> {
        HostedHttpClient::with_transport("https://api.example", &MockTransport::default())?;
        HostedHttpClient::with_transport("http://8.8.8.8", &MockTransport::default())?;
        Ok(())
    }

    #[test]
    #[cfg(feature = "async-http")]
    fn reqwest_transport_does_not_follow_redirects() -> Result<(), HostedHttpTestError> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let server = std::thread::spawn(move || -> Result<String, std::io::Error> {
            let (mut stream, _) = listener.accept()?;
            let mut buffer = [0_u8; 1024];
            let bytes_read = stream.read(&mut buffer)?;
            stream.write_all(
                b"HTTP/1.1 302 Found\r\nLocation: /redirected\r\nContent-Length: 0\r\n\r\n",
            )?;
            Ok(String::from_utf8_lossy(&buffer[..bytes_read]).into_owned())
        });

        let transport = ReqwestHttpTransport::with_private_network_access_for_tests()?;
        let response = transport.send(HostedHttpRequest {
            method: HttpMethod::Get,
            url: format!("http://{address}/start"),
            headers: Vec::new(),
            body: None,
        })?;
        let request = server
            .join()
            .map_err(|_| HostedHttpTestError::ServerThread)??;

        assert_eq!(response.status, 302);
        assert!(request.starts_with("GET /start "));
        Ok(())
    }

    #[test]
    #[cfg(feature = "async-http")]
    fn reqwest_transport_rejects_header_injection() -> Result<(), HostedHttpTestError> {
        let transport = ReqwestHttpTransport::new()?;
        let error = transport
            .send(HostedHttpRequest {
                method: HttpMethod::Get,
                url: "https://api.example/v1".to_owned(),
                headers: vec![HostedHttpHeader::new("x-runx", "good\nbad")],
                body: None,
            })
            .err();
        assert!(matches!(
            error,
            Some(HostedHttpError::InvalidHeaderValue { .. })
        ));
        Ok(())
    }

    #[cfg(feature = "async-http")]
    #[test]
    fn reqwest_transport_rejects_non_http_urls_before_sending() -> Result<(), HostedHttpTestError> {
        let transport = ReqwestHttpTransport::new()?;
        let error = transport
            .send(HostedHttpRequest {
                method: HttpMethod::Get,
                url: "file:///etc/passwd".to_owned(),
                headers: Vec::new(),
                body: None,
            })
            .err();

        assert!(matches!(
            error,
            Some(HostedHttpError::UnsupportedUrlScheme { .. })
        ));
        Ok(())
    }

    #[cfg(feature = "async-http")]
    #[test]
    fn reqwest_transport_times_out_stalled_response() -> Result<(), HostedHttpTestError> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let server = std::thread::spawn(move || -> Result<(), std::io::Error> {
            let (_stream, _) = listener.accept()?;
            std::thread::sleep(Duration::from_millis(500));
            Ok(())
        });

        let transport = ReqwestHttpTransport::with_private_network_timeouts_for_tests(
            Duration::from_millis(100),
            Duration::from_millis(100),
        )?;
        let error = transport
            .send(HostedHttpRequest {
                method: HttpMethod::Get,
                url: format!("http://{address}/stall"),
                headers: Vec::new(),
                body: None,
            })
            .err();
        server
            .join()
            .map_err(|_| HostedHttpTestError::ServerThread)??;

        assert!(matches!(error, Some(HostedHttpError::Transport { .. })));
        Ok(())
    }
}
