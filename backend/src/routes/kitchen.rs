use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;

use crate::auth::{resolve_tenant, tenant_selector, CurrentUser};
use crate::error::AppError;
use crate::models::{KitchenDeskView, KitchenItemView, KitchenOrderView};
use crate::response::ApiResponse;
use crate::state::AppState;

/// 排餐看板：已支付/制作中的订单按桌分组
/// 桌排序原则：按最早订单时间从早到晚（同桌优先出餐）
/// 桌内合并原则：菜品名+口味+备注 完全一致时合并数量，一次出餐
pub async fn kitchen_pending(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<KitchenDeskView>>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id = resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    // 1. 待出餐订单（先点先做）
    let order_rows = sqlx::query_as::<_, (i64, String, String, String, chrono::NaiveDateTime)>(
        r#"
SELECT id, order_no, desk_code, status, created_time
FROM food_order
WHERE tenant_id = $1 AND status IN ('PAID', 'PREPARING')
ORDER BY created_time ASC
"#,
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?;

    let order_ids: Vec<i64> = order_rows.iter().map(|row| row.0).collect();
    let mut item_map: HashMap<i64, Vec<KitchenItemView>> = HashMap::new();
    if !order_ids.is_empty() {
        let item_rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, i32)>(
            r#"
SELECT order_id, item_name, COALESCE(flavor, '') AS flavor, COALESCE(remark, '') AS remark, quantity
FROM food_order_item
WHERE order_id = ANY($1)
ORDER BY id ASC
"#,
        )
        .bind(&order_ids)
        .fetch_all(&state.pool)
        .await?;
        for (order_id, item_name, flavor, remark, quantity) in item_rows {
            item_map.entry(order_id).or_default().push(KitchenItemView {
                item_name,
                flavor: flavor.unwrap_or_default(),
                remark: remark.unwrap_or_default(),
                quantity,
            });
        }
    }

    // 2. 按桌分组（保持订单时间顺序，桌内首个订单时间即该桌最早下单时间）
    let mut desks: Vec<KitchenDeskView> = Vec::new();
    let mut desk_index: HashMap<String, usize> = HashMap::new();
    for (id, order_no, desk_code, status, created_time) in order_rows {
        let index = match desk_index.get(&desk_code) {
            Some(index) => *index,
            None => {
                desks.push(KitchenDeskView {
                    desk_code: desk_code.clone(),
                    order_count: 0,
                    earliest_at: created_time.format("%H:%M").to_string(),
                    orders: Vec::new(),
                    merged_items: Vec::new(),
                });
                desk_index.insert(desk_code.clone(), desks.len() - 1);
                desks.len() - 1
            }
        };
        desks[index].order_count += 1;
        desks[index].orders.push(KitchenOrderView {
            order_id: id,
            order_no,
            status,
            items: item_map.remove(&id).unwrap_or_default(),
        });
    }

    // 3. 桌内合并：相同菜品+口味+备注 → 合并数量（保持首次出现顺序）
    for desk in &mut desks {
        let mut merged: Vec<KitchenItemView> = Vec::new();
        let mut index_of: HashMap<(String, String, String), usize> = HashMap::new();
        for order in &desk.orders {
            for item in &order.items {
                let key = (
                    item.item_name.clone(),
                    item.flavor.clone(),
                    item.remark.clone(),
                );
                match index_of.get(&key) {
                    Some(i) => merged[*i].quantity += item.quantity,
                    None => {
                        index_of.insert(key, merged.len());
                        merged.push(item.clone());
                    }
                }
            }
        }
        desk.merged_items = merged;
    }

    Ok(ApiResponse::success(desks))
}
