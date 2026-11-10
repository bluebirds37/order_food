-- 多租户点餐系统基础表结构
-- 所有业务数据都通过 tenant_id 隔离

CREATE TABLE IF NOT EXISTS tenant (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(32) NOT NULL UNIQUE,
    name VARCHAR(64) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_time TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tenant_business_status (
    tenant_id BIGINT PRIMARY KEY REFERENCES tenant (id) ON DELETE CASCADE,
    open BOOLEAN NOT NULL DEFAULT FALSE,
    notice VARCHAR(255) NOT NULL DEFAULT '暂无公告',
    store_name VARCHAR(64) NOT NULL,
    intro VARCHAR(1024) NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS admin_account (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(32) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(64) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('SUPER_ADMIN', 'TENANT_ADMIN')),
    tenant_id BIGINT REFERENCES tenant (id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_time TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_admin_role_tenant CHECK (
        (role = 'SUPER_ADMIN' AND tenant_id IS NULL)
        OR (role = 'TENANT_ADMIN' AND tenant_id IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_admin_account_tenant ON admin_account (tenant_id);

CREATE TABLE IF NOT EXISTS customer_account (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    phone VARCHAR(32) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(64) NOT NULL,
    created_time TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, phone)
);

CREATE TABLE IF NOT EXISTS auth_session (
    token VARCHAR(64) PRIMARY KEY,
    account_type VARCHAR(20) NOT NULL CHECK (account_type IN ('CUSTOMER', 'TENANT_ADMIN', 'SUPER_ADMIN')),
    account_id BIGINT NOT NULL,
    tenant_id BIGINT REFERENCES tenant (id) ON DELETE CASCADE,
    created_time TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_time TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_auth_session_expires ON auth_session (expires_time);

CREATE TABLE IF NOT EXISTS menu_category (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (id, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_menu_category_tenant ON menu_category (tenant_id);

CREATE TABLE IF NOT EXISTS menu_item (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    category_id BIGINT NOT NULL,
    name VARCHAR(64) NOT NULL,
    price NUMERIC(10, 2) NOT NULL,
    description VARCHAR(512) NOT NULL DEFAULT '',
    image_url VARCHAR(512) NOT NULL DEFAULT '',
    recommended BOOLEAN NOT NULL DEFAULT FALSE,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (category_id, tenant_id) REFERENCES menu_category (id, tenant_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_menu_item_tenant ON menu_item (tenant_id);
CREATE INDEX IF NOT EXISTS idx_menu_item_category ON menu_item (category_id);

CREATE TABLE IF NOT EXISTS food_order (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    customer_id BIGINT REFERENCES customer_account (id) ON DELETE SET NULL,
    order_no VARCHAR(32) NOT NULL,
    customer_name VARCHAR(32) NOT NULL,
    customer_phone VARCHAR(32) NOT NULL,
    desk_code VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    total_amount NUMERIC(10, 2) NOT NULL,
    remark VARCHAR(255),
    created_time TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, order_no)
);

CREATE INDEX IF NOT EXISTS idx_food_order_tenant_created ON food_order (tenant_id, created_time DESC);
CREATE INDEX IF NOT EXISTS idx_food_order_customer_phone ON food_order (tenant_id, customer_phone);

CREATE TABLE IF NOT EXISTS food_order_item (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES food_order (id) ON DELETE CASCADE,
    menu_item_id BIGINT NOT NULL REFERENCES menu_item (id),
    item_name VARCHAR(64) NOT NULL,
    quantity INTEGER NOT NULL,
    price NUMERIC(10, 2) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_food_order_item_order ON food_order_item (order_id);
