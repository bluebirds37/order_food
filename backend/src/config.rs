use std::path::PathBuf;

use anyhow::{Context, Result};

/// 运行配置，全部来自环境变量或 .env 文件。
#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub image_dir: PathBuf,
    pub public_base_url: String,
    pub token_ttl_hours: i64,
    pub super_admin_username: String,
    pub super_admin_password: String,
    pub super_admin_display_name: String,
    pub initial_tenant_code: String,
    pub initial_tenant_name: String,
    pub initial_tenant_store_name: String,
    pub initial_tenant_notice: String,
    pub initial_tenant_intro: String,
    pub initial_tenant_admin_username: String,
    pub initial_tenant_admin_password: String,
    pub initial_tenant_admin_display_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .context("缺少 DATABASE_URL，请在 backend/.env 中配置 PostgreSQL 连接串")?;

        let port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("APP_PORT 必须是合法端口号")?;

        let image_dir = std::env::var("IMAGE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/images"));

        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{port}"))
            .trim_end_matches('/')
            .to_string();

        let token_ttl_hours = std::env::var("TOKEN_TTL_HOURS")
            .unwrap_or_else(|_| "168".to_string())
            .parse::<i64>()
            .context("TOKEN_TTL_HOURS 必须是整数")?;

        let super_admin_username = required("SUPER_ADMIN_USERNAME")?;
        let super_admin_password = required("SUPER_ADMIN_PASSWORD")?;
        let super_admin_display_name = env_or("SUPER_ADMIN_DISPLAY_NAME", "超级管理员");

        let initial_tenant_code = required("INITIAL_TENANT_CODE")?;
        let initial_tenant_name = required("INITIAL_TENANT_NAME")?;
        let initial_tenant_store_name = env_or("INITIAL_TENANT_STORE_NAME", &initial_tenant_name);
        let initial_tenant_notice = env_or("INITIAL_TENANT_NOTICE", "欢迎光临，现点现做");
        let initial_tenant_intro = env_or(
            "INITIAL_TENANT_INTRO",
            "精选时令食材，现点现做，欢迎扫码点餐。",
        );
        let initial_tenant_admin_username = required("INITIAL_TENANT_ADMIN_USERNAME")?;
        let initial_tenant_admin_password = required("INITIAL_TENANT_ADMIN_PASSWORD")?;
        let initial_tenant_admin_display_name =
            env_or("INITIAL_TENANT_ADMIN_DISPLAY_NAME", "门店管理员");

        Ok(Self {
            port,
            database_url,
            image_dir,
            public_base_url,
            token_ttl_hours,
            super_admin_username,
            super_admin_password,
            super_admin_display_name,
            initial_tenant_code,
            initial_tenant_name,
            initial_tenant_store_name,
            initial_tenant_notice,
            initial_tenant_intro,
            initial_tenant_admin_username,
            initial_tenant_admin_password,
            initial_tenant_admin_display_name,
        })
    }
}

fn required(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("缺少 {key}，请在 backend/.env 中配置"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
