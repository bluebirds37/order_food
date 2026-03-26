package com.example.orderfood.dto;

import com.example.orderfood.enums.OrderStatus;
import lombok.Builder;
import lombok.Data;

import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.util.List;

/**
 * 订单响应。
 */
@Data
@Builder
public class OrderResponse {

    private Long id;

    private String orderNo;

    private String customerName;

    private String customerPhone;

    private String deskCode;

    private OrderStatus status;

    private BigDecimal totalAmount;

    private String remark;

    private LocalDateTime createdTime;

    private List<OrderItemResponse> items;
}
