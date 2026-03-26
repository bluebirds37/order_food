package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

import java.math.BigDecimal;
import java.util.List;

/**
 * 看板响应。
 */
@Data
@Builder
public class DashboardResponse {

    private String storeName;

    private boolean open;

    private String notice;

    private long todayOrderCount;

    private BigDecimal todayRevenue;

    private long availableMenuCount;

    private List<OrderResponse> latestOrders;
}
