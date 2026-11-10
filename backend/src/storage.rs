use std::path::PathBuf;

use crate::error::AppError;
use crate::models::ImageView;

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

/// 图片统一存储。上传、覆盖、删除都通过这里处理，避免散落到多个目录。
#[derive(Clone, Debug)]
pub struct ImageStore {
    dir: PathBuf,
    public_base_url: String,
}

impl ImageStore {
    pub fn new(dir: PathBuf, public_base_url: String) -> Self {
        Self {
            dir,
            public_base_url,
        }
    }

    pub async fn ensure_dir(&self) -> Result<(), AppError> {
        tokio::fs::create_dir_all(&self.dir).await?;
        Ok(())
    }

    /// 保存图片。`preferred_filename` 为空时使用 UUID 生成文件名。
    /// `overwrite=true` 用于覆盖已有图片。
    pub async fn save(
        &self,
        preferred_filename: Option<&str>,
        original_filename: Option<&str>,
        content_type: Option<&str>,
        bytes: &[u8],
        overwrite: bool,
    ) -> Result<ImageView, AppError> {
        if bytes.is_empty() {
            return Err(AppError::business("图片内容不能为空"));
        }
        if bytes.len() > MAX_IMAGE_SIZE {
            return Err(AppError::business("图片大小不能超过 10MB"));
        }

        let (desired_stem, desired_ext) = match preferred_filename {
            Some(name) if !name.trim().is_empty() => split_filename(name).unwrap_or((name, "")),
            _ => ("", ""),
        };
        let (_, original_ext) = original_filename
            .and_then(split_filename)
            .unwrap_or(("", ""));

        let extension = pick_extension(&[desired_ext, original_ext], content_type)?;
        let stem = if !desired_stem.is_empty() {
            sanitize_stem(desired_stem)?
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        let filename = format!("{stem}.{extension}");

        let path = self.dir.join(&filename);
        if path.exists() && !overwrite {
            return Err(AppError::business("图片已存在，如需覆盖请使用覆盖接口"));
        }

        tokio::fs::write(&path, bytes).await?;
        tracing::info!(?path, size = bytes.len(), "图片已保存");
        Ok(self.to_view(&filename))
    }

    /// 删除图片。文件不存在时同样返回成功，便于幂等调用。
    pub async fn delete(&self, filename: &str) -> Result<(), AppError> {
        let path = self.resolve(filename)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {
                tracing::info!(?path, "图片已删除");
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::from(error)),
        }
    }

    /// 保存后端生成的资源（例如租户二维码），同样写入统一图片目录。
    pub async fn save_generated(&self, filename: &str, bytes: &[u8]) -> Result<ImageView, AppError> {
        let (stem, ext) =
            split_filename(filename).ok_or_else(|| AppError::business("图片文件名不合法"))?;
        let stem = sanitize_stem(stem)?;
        let ext = ext.to_ascii_lowercase();
        if !matches!(ext.as_str(), "svg" | "png" | "jpg" | "jpeg" | "webp") {
            return Err(AppError::business("不支持的生成图片格式"));
        }
        let filename = format!("{stem}.{ext}");
        let path = self.dir.join(&filename);
        tokio::fs::write(&path, bytes).await?;
        tracing::info!(?path, size = bytes.len(), "生成图片已保存");
        Ok(self.to_view(&filename))
    }

    /// 将外部传入的文件名规范化成实际落盘的文件名，供归属校验与文件操作共用。
    pub fn normalize_filename(&self, filename: &str) -> Result<String, AppError> {
        let (stem, ext) =
            split_filename(filename).ok_or_else(|| AppError::business("图片文件名不合法"))?;
        let stem = sanitize_stem(stem)?;
        let ext = ext.to_ascii_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
            return Err(AppError::business(
                "仅支持 jpg、jpeg、png、gif、webp、bmp 格式",
            ));
        }
        Ok(format!("{stem}.{ext}"))
    }

    pub fn exists(&self, filename: &str) -> Result<bool, AppError> {
        Ok(self.resolve(filename)?.exists())
    }

    fn resolve(&self, filename: &str) -> Result<PathBuf, AppError> {
        Ok(self.dir.join(self.normalize_filename(filename)?))
    }

    fn to_view(&self, filename: &str) -> ImageView {
        ImageView {
            filename: filename.to_string(),
            path: format!("/images/{filename}"),
            url: format!("{}/images/{filename}", self.public_base_url),
        }
    }
}

fn split_filename(name: &str) -> Option<(&str, &str)> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() {
        return None;
    }
    Some((stem, ext))
}

fn sanitize_stem(stem: &str) -> Result<String, AppError> {
    let stem = stem.trim();
    if stem.is_empty() || stem.len() > 120 {
        return Err(AppError::business("图片文件名不合法"));
    }
    if !stem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::business(
            "图片文件名仅支持字母、数字、短横线和下划线",
        ));
    }
    Ok(stem.to_string())
}

fn pick_extension(extensions: &[&str], content_type: Option<&str>) -> Result<String, AppError> {
    for extension in extensions {
        let extension = extension.trim().to_ascii_lowercase();
        if !extension.is_empty() && ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            return Ok(extension);
        }
    }

    let extension = match content_type {
        Some(value) if value.starts_with("image/jpeg") => "jpg",
        Some(value) if value.starts_with("image/png") => "png",
        Some(value) if value.starts_with("image/gif") => "gif",
        Some(value) if value.starts_with("image/webp") => "webp",
        Some(value) if value.starts_with("image/bmp") => "bmp",
        _ => {
            return Err(AppError::business(
                "仅支持 jpg、jpeg、png、gif、webp、bmp 格式",
            ))
        }
    };
    Ok(extension.to_string())
}
