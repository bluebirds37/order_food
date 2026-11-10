use axum::extract::multipart::{Multipart, MultipartRejection};
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::auth::{resolve_tenant, CurrentUser};
use crate::error::AppError;
use crate::models::ImageView;
use crate::response::ApiResponse;
use crate::state::AppState;

/// 上传时的租户选择：租户管理员固定使用自己所属租户；
/// 超级管理员可指定租户，未指定时记为平台级资源（tenant_id 为 NULL）。
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTenantQuery {
    tenant_id: Option<i64>,
    tenant_code: Option<String>,
}

pub async fn upload(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ImageTenantQuery>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Json<ApiResponse<ImageView>>, AppError> {
    auth.require_admin()?;
    let owner = resolve_image_tenant(&state, &auth, &query).await?;
    let multipart = multipart.map_err(|_| AppError::bad_request())?;
    let upload = read_upload(multipart).await?;

    let image = state
        .images
        .save(
            upload.preferred_filename.as_deref(),
            upload.original_filename.as_deref(),
            upload.content_type.as_deref(),
            &upload.bytes,
            false,
        )
        .await?;

    record_owner(&state, &auth, &image.filename, owner).await?;
    Ok(ApiResponse::success_with_message("上传成功", image))
}

pub async fn overwrite(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(filename): Path<String>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Json<ApiResponse<ImageView>>, AppError> {
    auth.require_admin()?;
    let filename = state.images.normalize_filename(&filename)?;
    ensure_can_modify(&state, &auth, &filename).await?;

    let multipart = multipart.map_err(|_| AppError::bad_request())?;
    let upload = read_upload(multipart).await?;

    let image = state
        .images
        .save(
            Some(&filename),
            upload.original_filename.as_deref(),
            upload.content_type.as_deref(),
            &upload.bytes,
            true,
        )
        .await?;

    // 覆盖不改变已有归属；只有原本没有归属记录时才补记当前上传者。
    record_owner_if_absent(&state, &auth, &image.filename).await?;
    Ok(ApiResponse::success_with_message("覆盖成功", image))
}

pub async fn remove(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth.require_admin()?;
    let filename = state.images.normalize_filename(&filename)?;
    ensure_can_modify(&state, &auth, &filename).await?;

    state.images.delete(&filename).await?;
    sqlx::query("DELETE FROM image_asset WHERE filename = $1")
        .bind(&filename)
        .execute(&state.pool)
        .await?;
    Ok(ApiResponse::<()>::success_empty("删除成功"))
}

async fn resolve_image_tenant(
    state: &AppState,
    auth: &CurrentUser,
    query: &ImageTenantQuery,
) -> Result<Option<i64>, AppError> {
    if auth.is_super_admin()
        && query.tenant_id.is_none()
        && query
            .tenant_code
            .as_deref()
            .map(str::trim)
            .is_none_or(str::is_empty)
    {
        return Ok(None);
    }
    let tenant_id = resolve_tenant(
        state,
        auth,
        query.tenant_id,
        query.tenant_code.as_deref(),
    )
    .await?;
    Ok(Some(tenant_id))
}

async fn ensure_can_modify(
    state: &AppState,
    auth: &CurrentUser,
    filename: &str,
) -> Result<(), AppError> {
    if auth.is_super_admin() {
        return Ok(());
    }

    let owner = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT tenant_id FROM image_asset WHERE filename = $1",
    )
    .bind(filename)
    .fetch_optional(&state.pool)
    .await?;

    match owner {
        Some(Some(tenant_id)) if Some(tenant_id) == auth.tenant_id => Ok(()),
        Some(_) => Err(AppError::forbidden("无权操作其他租户的图片")),
        // 没有归属记录但文件存在（二维码或历史图片），租户管理员不能操作。
        None if state.images.exists(filename)? => Err(AppError::forbidden("无权操作其他租户的图片")),
        None => Ok(()),
    }
}

async fn record_owner(
    state: &AppState,
    auth: &CurrentUser,
    filename: &str,
    tenant_id: Option<i64>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
INSERT INTO image_asset (filename, tenant_id, uploaded_by)
VALUES ($1, $2, $3)
ON CONFLICT (filename) DO UPDATE
SET tenant_id = EXCLUDED.tenant_id,
    uploaded_by = EXCLUDED.uploaded_by
"#,
    )
    .bind(filename)
    .bind(tenant_id)
    .bind(auth.account_id)
    .execute(&state.pool)
    .await?;
    Ok(())
}

async fn record_owner_if_absent(
    state: &AppState,
    auth: &CurrentUser,
    filename: &str,
) -> Result<(), AppError> {
    let tenant_id = if auth.is_super_admin() {
        None
    } else {
        auth.tenant_id
    };
    sqlx::query(
        r#"
INSERT INTO image_asset (filename, tenant_id, uploaded_by)
VALUES ($1, $2, $3)
ON CONFLICT (filename) DO NOTHING
"#,
    )
    .bind(filename)
    .bind(tenant_id)
    .bind(auth.account_id)
    .execute(&state.pool)
    .await?;
    Ok(())
}

struct Upload {
    preferred_filename: Option<String>,
    original_filename: Option<String>,
    content_type: Option<String>,
    bytes: Vec<u8>,
}

async fn read_upload(mut multipart: Multipart) -> Result<Upload, AppError> {
    let mut preferred_filename = None;
    let mut original_filename = None;
    let mut content_type = None;
    let mut bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request())?
    {
        let field_name = field.name().map(str::to_string);
        match field_name.as_deref() {
            Some("filename") => {
                preferred_filename = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| AppError::bad_request())?
                        .trim()
                        .to_string(),
                );
            }
            Some("file") => {
                original_filename = field.file_name().map(str::to_string);
                content_type = field.content_type().map(str::to_string);
                bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::bad_request())?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let bytes = bytes.ok_or_else(|| AppError::business("请选择要上传的图片"))?;
    Ok(Upload {
        preferred_filename,
        original_filename,
        content_type,
        bytes,
    })
}
