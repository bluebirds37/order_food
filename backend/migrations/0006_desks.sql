-- 桌号管理：门店可维护桌号，顾客端以下拉选择桌号
CREATE TABLE IF NOT EXISTS desk (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenant(id),
    desk_code VARCHAR(16) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, desk_code)
);

-- 为现有启用租户预置常见桌号 A01-A06（管理员可自行增删改）
INSERT INTO desk (tenant_id, desk_code)
SELECT t.id, 'A' || lpad(g::text, 2, '0')
FROM tenant t
CROSS JOIN generate_series(1, 6) g
WHERE t.enabled
ON CONFLICT DO NOTHING;
