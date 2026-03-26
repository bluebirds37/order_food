package com.example.orderfood.dto;

import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;
import lombok.Data;

/**
 * 订单明细请求。
 */
@Data
public class CreateOrderItemRequest {

    @NotNull(message = "菜品不能为空")
    private Long menuItemId;

    @NotNull(message = "数量不能为空")
    @Min(value = 1, message = "数量至少为1")
    private Integer quantity;
}
