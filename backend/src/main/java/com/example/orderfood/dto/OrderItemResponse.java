package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

import java.math.BigDecimal;

/**
 * 订单明细响应。
 */
@Data
@Builder
public class OrderItemResponse {

    private Long menuItemId;

    private String itemName;

    private Integer quantity;

    private BigDecimal price;
}
