package com.example.orderfood.service;

import com.example.orderfood.dto.CreateOrderRequest;
import com.example.orderfood.dto.OrderResponse;
import com.example.orderfood.dto.OrderStatusRequest;

import java.util.List;

/**
 * 订单服务。
 */
public interface OrderService {

    OrderResponse createOrder(CreateOrderRequest request);

    OrderResponse getOrderDetail(Long orderId);

    OrderResponse getOrderByOrderNo(String orderNo);

    List<OrderResponse> getOrdersByPhone(String phone);

    List<OrderResponse> getAllOrders();

    OrderResponse updateOrderStatus(Long orderId, OrderStatusRequest request);
}
