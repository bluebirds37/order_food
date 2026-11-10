-- 菜品详情：图片 banner（JSON 数组），详情文字复用 description 字段
ALTER TABLE menu_item
    ADD COLUMN IF NOT EXISTS detail_images JSONB NOT NULL DEFAULT '[]'::jsonb;
