-- 菜品口味选项（JSON 字符串数组）+ 订单明细口味/备注
ALTER TABLE menu_item
    ADD COLUMN IF NOT EXISTS options JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE food_order_item
    ADD COLUMN IF NOT EXISTS flavor TEXT,
    ADD COLUMN IF NOT EXISTS remark TEXT;
