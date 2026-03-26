package com.example.orderfood.dto;

import com.example.orderfood.enums.OrderStatus;
import jakarta.validation.constraints.NotNull;
import lombok.Data;

/**
 * 更新订单状态请求。
 */
@Data
public class OrderStatusRequest {

    @NotNull(message = "订单状态不能为空")
    private OrderStatus status;
}
