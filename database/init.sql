-- 微信点餐系统数据库初始化脚本
CREATE TABLE IF NOT EXISTS admin_user (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(32) NOT NULL UNIQUE,
    password VARCHAR(64) NOT NULL,
    display_name VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS business_status (
    id BIGINT PRIMARY KEY,
    open BOOLEAN NOT NULL,
    notice VARCHAR(255),
    store_name VARCHAR(64)
);

INSERT INTO admin_user (username, password, display_name)
SELECT 'admin', 'admin', '门店老板'
WHERE NOT EXISTS (SELECT 1 FROM admin_user WHERE username = 'admin');

INSERT INTO business_status (id, open, notice, store_name)
SELECT 1, TRUE, '欢迎光临，现点现做', '食光小馆'
WHERE NOT EXISTS (SELECT 1 FROM business_status WHERE id = 1);
