use chrono::{DateTime, Utc};
use native_tls::TlsConnector;
use std::borrow::Cow::{self, Borrowed};
use std::net::TcpStream;
use tabled::Tabled;

#[derive(Debug)]
pub struct CertificateInfo {
    domain: String,
    issuer: String,
    valid_from: DateTime<Utc>,
    valid_to: DateTime<Utc>,
    days_remaining: i64,
}

impl Tabled for CertificateInfo {
    const LENGTH: usize = 4;
    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            Borrowed("域名"),
            Borrowed("签发时间"),
            Borrowed("签发机构"),
            Borrowed("到期时间"),
            Borrowed("剩余天数"),
        ]
    }
    fn fields(&self) -> Vec<Cow<'_, str>> {
        let valid_from = self.valid_from.format("%Y-%m-%d %H:%M").to_string();
        let valid_to = self.valid_to.format("%Y-%m-%d %H:%M").to_string();
        vec![
            self.domain.as_str().into(),
            valid_from.into(),
            self.issuer.as_str().into(),
            valid_to.into(),
            self.days_remaining.to_string().into(),
        ]
    }
}

impl CertificateInfo {
    pub fn days_remaining(&self) -> i64 {
        self.days_remaining
    }

    pub fn need_update(&self) -> bool {
        self.days_remaining <= 3
    }
}

pub fn check_ssl_certificate(domain: &str) -> crate::Result<CertificateInfo> {
    // 尝试使用HTTPS
    let connector = TlsConnector::new()?;

    // 连接到服务器
    let stream = TcpStream::connect(format!("{}:443", domain))?;
    let tls_stream = connector.connect(domain, stream)?;

    // 获取证书
    let cert = tls_stream.peer_certificate()?.ok_or("无法获取SSL证书")?;

    // 获取证书的DER编码
    let der = cert.to_der()?;

    // 解析证书
    let parsed_cert =
        x509_parser::parse_x509_certificate(&der).map_err(|e| format!("解析证书失败: {}", e))?;

    let cert = parsed_cert.1;

    // 获取有效期
    let valid_from =
        DateTime::from_timestamp(cert.validity().not_before.timestamp(), 0).unwrap_or(Utc::now());
    let valid_to =
        DateTime::from_timestamp(cert.validity().not_after.timestamp(), 0).unwrap_or(Utc::now());
    let days_remaining = (valid_to - Utc::now()).num_days();

    Ok(CertificateInfo {
        domain: domain.to_string(),
        issuer: cert.issuer().to_string(),
        valid_from,
        valid_to,
        days_remaining,
    })
}
