use axum::extract::{Path, State};
use axum::Json;
use qrcode::{Color, QrCode};

use crate::auth::{find_tenant_by_code, qr_content};
use crate::error::AppError;
use crate::models::{ImageView, TenantEntryView};
use crate::response::ApiResponse;
use crate::routes::business::load_status;
use crate::state::AppState;

/// 扫二维码进入时的租户校验接口（公开）。
pub async fn entry(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<TenantEntryView>>, AppError> {
    let tenant = find_tenant_by_code(&state.pool, &code).await?;
    if !tenant.enabled {
        return Err(AppError::business("该门店已暂停服务"));
    }
    let status = load_status(&state.pool, tenant.id).await?;
    Ok(ApiResponse::success(TenantEntryView {
        tenant_id: tenant.id,
        code: tenant.code,
        name: tenant.name,
        store_name: status.store_name,
        intro: status.intro,
        open: status.open,
        notice: status.notice,
    }))
}

/// 生成租户实体二维码 PNG，二维码内容携带租户编码，图片写入统一图片目录。
pub async fn generate_tenant_qr(
    state: &AppState,
    tenant_code: &str,
) -> Result<ImageView, AppError> {
    let content = qr_content(tenant_code);
    let bytes = render_qr_png(&content)?;
    let filename = format!("tenant-{}-qrcode.png", tenant_code.to_ascii_lowercase());
    state.images.save_generated(&filename, &bytes).await
}

fn render_qr_png(content: &str) -> Result<Vec<u8>, AppError> {
    let code = QrCode::new(content.as_bytes()).map_err(|error| {
        tracing::error!(error = %error, "二维码生成失败");
        AppError::internal()
    })?;
    let modules = code.width();
    let colors = code.to_colors();
    let quiet_zone = 4usize;
    let scale = 8usize;
    let size = (modules + quiet_zone * 2) * scale;
    let mut pixels = vec![255u8; size * size];

    for y in 0..modules {
        for x in 0..modules {
            if colors[y * modules + x] == Color::Dark {
                let start_x = (x + quiet_zone) * scale;
                let start_y = (y + quiet_zone) * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        pixels[(start_y + dy) * size + start_x + dx] = 0;
                    }
                }
            }
        }
    }

    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, size as u32, size as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| {
            tracing::error!(error = %error, "二维码 PNG 编码失败");
            AppError::internal()
        })?;
        writer.write_image_data(&pixels).map_err(|error| {
            tracing::error!(error = %error, "二维码 PNG 写入失败");
            AppError::internal()
        })?;
    }
    Ok(png_bytes)
}

