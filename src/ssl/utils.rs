use crate::Result;
use crate::error::AppError;
use base64::prelude::*;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::NamedTempFile;
use tracing::info;
use zip::ZipArchive;

#[derive(Debug)]
pub struct CertFile {
    pub public_key: String,
    pub private_key: String,
}

pub fn parse_cert_from_base64(content: &str) -> Result<CertFile> {
    let zip_data = BASE64_STANDARD.decode(content)?;
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(&zip_data)?;
    temp_file.seek(SeekFrom::Start(0))?;
    let mut zip_archive = ZipArchive::new(&temp_file)?;

    let mut public_key = String::new();
    let mut private_key = String::new();

    for i in 0..zip_archive.len() {
        if !public_key.is_empty() && !private_key.is_empty() {
            return Ok(CertFile {
                public_key,
                private_key,
            });
        }
        let mut file = zip_archive.by_index(i)?;

        let file_name = file.name().to_lowercase();
        info!("file name:{}", file_name);
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let data = String::from_utf8_lossy(&data);
        // info!("file data:{}",&data);
        if data.contains("--BEGIN CERTIFICATE--") {
            public_key = data.to_string();
        }
        if data.contains("-BEGIN RSA PRIVATE KEY--") {
            private_key = data.to_string();
        }
    }
    Err(AppError::Other("file is not valid".to_string()))
}
