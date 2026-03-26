package com.example.orderfood.service.impl;

import com.example.orderfood.dto.OrderItemResponse;
import com.example.orderfood.dto.OrderResponse;
import com.example.orderfood.entity.FoodOrder;

/**
 * 订单对象组装器。
 */
public class OrderAssembler {

    public OrderResponse toResponse(FoodOrder order) {
        return OrderResponse.builder()
                .id(order.getId())
                .orderNo(order.getOrderNo())
                .customerName(order.getCustomerName())
                .customerPhone(order.getCustomerPhone())
                .deskCode(order.getDeskCode())
                .status(order.getStatus())
                .totalAmount(order.getTotalAmount())
                .remark(order.getRemark())
                .createdTime(order.getCreatedTime())
                .items(order.getItems().stream()
                        .map(item -> OrderItemResponse.builder()
                                .menuItemId(item.getMenuItem().getId())
                                .itemName(item.getItemName())
                                .quantity(item.getQuantity())
                                .price(item.getPrice())
                                .build())
                        .toList())
                .build();
    }
}
