use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AllConfig {
    pub tencent_cloud: TencentCloudConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TencentCloudConfig {
    pub secret_id: String,
    pub secret_key: String,
}

pub fn get_all_config(config_path: &str) -> crate::Result<AllConfig> {
    let config_builder = Config::builder()
        // 加载配置文件
        .add_source(File::with_name(config_path))
        .build()?;

    let config = config_builder.try_deserialize()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File as StdFile;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_all_config() {
        // 创建一个临时目录
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.toml");

        // 构造一个配置文件内容
        let config_content = r#"
            [tencent_cloud]
            secret_id = "testid"
            secret_key = "testkey"
        "#;

        // 写入临时配置文件
        let mut file = StdFile::create(&file_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        // 调用get_all_config
        let config = get_all_config(file_path.to_str().unwrap()).unwrap();

        // 断言配置内容

        assert_eq!(config.tencent_cloud.secret_id, "testid");
        assert_eq!(config.tencent_cloud.secret_key, "testkey");
    }
}
