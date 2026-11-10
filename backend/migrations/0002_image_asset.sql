-- 图片归属记录：文件仍统一存放在 IMAGE_DIR，这里只记录每个文件属于哪个租户。
-- tenant_id 为 NULL 表示平台级资源（例如超级管理员生成的租户二维码）。
CREATE TABLE IF NOT EXISTS image_asset (
    filename VARCHAR(160) PRIMARY KEY,
    tenant_id BIGINT REFERENCES tenant (id) ON DELETE CASCADE,
    uploaded_by BIGINT,
    created_time TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_image_asset_tenant ON image_asset (tenant_id);
