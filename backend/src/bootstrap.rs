use anyhow::Result;
use sqlx::PgPool;

use crate::auth::hash_password;
use crate::config::Config;

/// 首次启动时创建初始租户、租户管理员与超级管理员。
/// 已存在的数据不会被覆盖，可安全重复执行。
pub async fn bootstrap(pool: &PgPool, config: &Config) -> Result<()> {
    let tenant_id = ensure_initial_tenant(pool, config).await?;
    ensure_super_admin(pool, config).await?;
    ensure_tenant_admin(pool, config, tenant_id).await?;
    Ok(())
}

async fn ensure_initial_tenant(pool: &PgPool, config: &Config) -> Result<i64> {
    let existing = sqlx::query_scalar::<_, i64>("SELECT id FROM tenant WHERE code = $1")
        .bind(&config.initial_tenant_code)
        .fetch_optional(pool)
        .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let mut tx = pool.begin().await?;
    let tenant_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO tenant (code, name, enabled) VALUES ($1, $2, TRUE) RETURNING id",
    )
    .bind(&config.initial_tenant_code)
    .bind(&config.initial_tenant_name)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
INSERT INTO tenant_business_status (tenant_id, open, notice, store_name, intro)
VALUES ($1, TRUE, $2, $3, $4)
"#,
    )
    .bind(tenant_id)
    .bind(&config.initial_tenant_notice)
    .bind(&config.initial_tenant_store_name)
    .bind(&config.initial_tenant_intro)
    .execute(&mut *tx)
    .await?;

    let flagship_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO menu_category (tenant_id, name, sort_order) VALUES ($1, '招牌热销', 1) RETURNING id",
    )
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await?;
    let drink_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO menu_category (tenant_id, name, sort_order) VALUES ($1, '饮品小食', 2) RETURNING id",
    )
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await?;

    let items = [
        (flagship_id, "黑椒牛肉饭", 28.00, "嫩煎牛肉搭配流心温泉蛋，口感浓郁。", true, 1),
        (flagship_id, "香酥鸡腿饭", 24.00, "外酥里嫩，搭配时蔬与秘制酱汁。", true, 2),
        (drink_id, "青柠气泡水", 12.00, "清爽解腻，适合搭配主食。", false, 1),
    ];
    for (category_id, name, price, description, recommended, sort_order) in items {
        sqlx::query(
            r#"
INSERT INTO menu_item
    (tenant_id, category_id, name, price, description, image_url, recommended, available, sort_order)
VALUES ($1, $2, $3, $4, $5, '', $6, TRUE, $7)
"#,
        )
        .bind(tenant_id)
        .bind(category_id)
        .bind(name)
        .bind(price)
        .bind(description)
        .bind(recommended)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(tenant_code = %config.initial_tenant_code, "已创建初始租户");
    Ok(tenant_id)
}

async fn ensure_super_admin(pool: &PgPool, config: &Config) -> Result<()> {
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM admin_account WHERE username = $1)")
            .bind(&config.super_admin_username)
            .fetch_one(pool)
            .await?;
    if exists {
        return Ok(());
    }

    let password_hash = hash_password(&config.super_admin_password)
        .map_err(|error| anyhow::anyhow!(error.message))?;
    sqlx::query(
        r#"
INSERT INTO admin_account (username, password_hash, display_name, role, tenant_id, enabled)
VALUES ($1, $2, $3, 'SUPER_ADMIN', NULL, TRUE)
"#,
    )
    .bind(&config.super_admin_username)
    .bind(password_hash)
    .bind(&config.super_admin_display_name)
    .execute(pool)
    .await?;
    tracing::info!(username = %config.super_admin_username, "已创建超级管理员");
    Ok(())
}

async fn ensure_tenant_admin(pool: &PgPool, config: &Config, tenant_id: i64) -> Result<()> {
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM admin_account WHERE username = $1)")
            .bind(&config.initial_tenant_admin_username)
            .fetch_one(pool)
            .await?;
    if exists {
        return Ok(());
    }

    let password_hash = hash_password(&config.initial_tenant_admin_password)
        .map_err(|error| anyhow::anyhow!(error.message))?;
    sqlx::query(
        r#"
INSERT INTO admin_account (username, password_hash, display_name, role, tenant_id, enabled)
VALUES ($1, $2, $3, 'TENANT_ADMIN', $4, TRUE)
"#,
    )
    .bind(&config.initial_tenant_admin_username)
    .bind(password_hash)
    .bind(&config.initial_tenant_admin_display_name)
    .bind(tenant_id)
    .execute(pool)
    .await?;
    tracing::info!(username = %config.initial_tenant_admin_username, "已创建租户管理员");
    Ok(())
}
