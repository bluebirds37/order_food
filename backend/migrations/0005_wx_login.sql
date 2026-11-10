-- 微信无感登录：顾客账号绑定 openid（openid 登录时手机号可为占位，待一键绑定）
ALTER TABLE customer_account
    ADD COLUMN IF NOT EXISTS openid VARCHAR(64);

CREATE UNIQUE INDEX IF NOT EXISTS uk_customer_tenant_openid
    ON customer_account (tenant_id, openid)
    WHERE openid IS NOT NULL;
