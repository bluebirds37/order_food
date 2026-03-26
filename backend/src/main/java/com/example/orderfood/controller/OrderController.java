package com.example.orderfood.controller;

import com.example.orderfood.common.ApiResponse;
import com.example.orderfood.dto.CreateOrderRequest;
import com.example.orderfood.dto.OrderResponse;
import com.example.orderfood.dto.OrderStatusRequest;
import com.example.orderfood.service.OrderService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

/**
 * 订单接口。
 */
@RestController
@RequiredArgsConstructor
@RequestMapping("/api/orders")
public class OrderController {

    private final OrderService orderService;

    @PostMapping
    public ApiResponse<OrderResponse> createOrder(@Valid @RequestBody CreateOrderRequest request) {
        return ApiResponse.success("下单成功", orderService.createOrder(request));
    }

    @GetMapping("/{orderId}")
    public ApiResponse<OrderResponse> getOrderDetail(@PathVariable Long orderId) {
        return ApiResponse.success(orderService.getOrderDetail(orderId));
    }

    @GetMapping("/query")
    public ApiResponse<OrderResponse> getByOrderNo(@RequestParam String orderNo) {
        return ApiResponse.success(orderService.getOrderByOrderNo(orderNo));
    }

    @GetMapping("/history")
    public ApiResponse<List<OrderResponse>> getHistory(@RequestParam String phone) {
        return ApiResponse.success(orderService.getOrdersByPhone(phone));
    }

    @GetMapping
    public ApiResponse<List<OrderResponse>> getAllOrders() {
        return ApiResponse.success(orderService.getAllOrders());
    }

    @PutMapping("/{orderId}/status")
    public ApiResponse<OrderResponse> updateStatus(@PathVariable Long orderId, @Valid @RequestBody OrderStatusRequest request) {
        return ApiResponse.success("状态更新成功", orderService.updateOrderStatus(orderId, request));
    }
}
