-- 创建本项目独立数据库（幂等）
-- 用法：
--   psql -h 127.0.0.1 -p 15432 -U postgres -f database/create_database.sql

SELECT 'CREATE DATABASE order_food'
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'order_food')
\gexec
