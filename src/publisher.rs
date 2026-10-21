use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use thiserror::Error;

use crate::WeatherRenderOutput;
use crate::provider::{env_bool, env_or, env_parse, load_dotenv_if_present};

pub const DEFAULT_DEVICE_ID: &str = "";
pub const DEFAULT_WEATHER_PAGE_ID: u8 = 2;
pub const CODEX_RESERVED_PAGE_ID: u8 = 3;

#[derive(Clone, Debug)]
pub struct ZectrixPublisherConfig {
    pub base_url: String,
    pub device_id: String,
    pub page_id: u8,
    pub reserved_pages: Vec<u8>,
    pub allow_reserved_override: bool,
    pub dither: bool,
    pub timeout_secs: u64,
    pub max_image_bytes: usize,
}

impl ZectrixPublisherConfig {
    pub fn from_env() -> Self {
        load_dotenv_if_present();
        let reserved_raw = env_or("ZECTRIX_RESERVED_PAGES", "3");
        let reserved_pages: Vec<u8> = reserved_raw
            .split(',')
            .filter_map(|s| s.trim().parse::<u8>().ok())
            .collect();
        let allow_reserved_override = env_bool("ALLOW_PAGE_3_OVERRIDE", false)
            || env_bool("ZECTRIX_ALLOW_RESERVED_OVERRIDE", false);

        Self {
            base_url: env_or("ZECTRIX_API_BASE", "https://cloud.zectrix.com"),
            device_id: env_or("ZECTRIX_DEVICE_ID", DEFAULT_DEVICE_ID),
            page_id: env_parse("ZECTRIX_PAGE_ID", DEFAULT_WEATHER_PAGE_ID),
            reserved_pages,
            allow_reserved_override,
            dither: env_bool("ZECTRIX_DITHER", false),
            timeout_secs: env_parse("ZECTRIX_HTTP_TIMEOUT_SECS", 30_u64).max(5),
            max_image_bytes: env_parse("ZECTRIX_MAX_IMAGE_BYTES", 2 * 1024 * 1024_usize),
        }
    }
}

#[derive(Debug, Error)]
pub enum ZectrixPublishError {
    #[error("ZECTRIX 发布配置无效：{0}")]
    InvalidConfiguration(String),
    #[error("无法编码气象站 PNG 图像")]
    Image,
    #[error("ZECTRIX 云端接口暂时不可用")]
    Unavailable,
}

pub fn validate_weather_page_id(page_id: u8, _allow_page_3: bool) -> Result<u8, ZectrixPublishError> {
    if !(1..=5).contains(&page_id) {
        return Err(ZectrixPublishError::InvalidConfiguration(
            "ZECTRIX_PAGE_ID 必须在 1 到 5 之间".into(),
        ));
    }
    Ok(page_id)
}

pub trait FramePublisher {
    type Error;

    fn publish(&mut self, output: &WeatherRenderOutput) -> Result<(), Self::Error>;
}

pub struct ZectrixPublisher {
    client: Client,
    cfg: ZectrixPublisherConfig,
}

impl ZectrixPublisher {
    pub fn new(
        api_key: &str,
        base_url: impl Into<String>,
        device_id: impl Into<String>,
        page_id: u8,
        allow_page_3: bool,
    ) -> Result<Self, ZectrixPublishError> {
        let mut cfg = ZectrixPublisherConfig::from_env();
        cfg.base_url = base_url.into().trim_end_matches('/').to_owned();
        cfg.device_id = device_id.into();
        cfg.page_id = validate_weather_page_id(page_id, allow_page_3)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-API-Key",
            HeaderValue::from_str(api_key.trim()).map_err(|_| {
                ZectrixPublishError::InvalidConfiguration("X-API-Key 格式无效".into())
            })?,
        );
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
            .build()
            .map_err(|_| {
                ZectrixPublishError::InvalidConfiguration("HTTP 客户端初始化失败".into())
            })?;
        Ok(Self { client, cfg })
    }
}

#[derive(Deserialize)]
struct ApiCodeResponse {
    code: i64,
}

impl FramePublisher for ZectrixPublisher {
    type Error = ZectrixPublishError;

    fn publish(&mut self, output: &WeatherRenderOutput) -> Result<(), Self::Error> {
        let image_bytes = output
            .frame
            .png_bytes()
            .map_err(|_| ZectrixPublishError::Image)?;
        if image_bytes.len() > self.cfg.max_image_bytes {
            return Err(ZectrixPublishError::Image);
        }
        let image_part = Part::bytes(image_bytes)
            .file_name("zectrix-weather.png")
            .mime_str("image/png")
            .map_err(|_| ZectrixPublishError::Image)?;
        let form = Form::new()
            .text("dither", self.cfg.dither.to_string())
            .text("pageId", self.cfg.page_id.to_string())
            .part("images", image_part);
        let response = self
            .client
            .post(format!(
                "{}/open/v1/devices/{}/display/image",
                self.cfg.base_url, self.cfg.device_id
            ))
            .multipart(form)
            .send()
            .map_err(|_| ZectrixPublishError::Unavailable)?;
        if !response.status().is_success() {
            return Err(ZectrixPublishError::Unavailable);
        }
        let payload: ApiCodeResponse = response
            .json()
            .map_err(|_| ZectrixPublishError::Unavailable)?;
        if payload.code != 0 {
            return Err(ZectrixPublishError::Unavailable);
        }
        Ok(())
    }
}
