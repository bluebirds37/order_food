package com.example.orderfood.service.impl;

import com.example.orderfood.dto.BusinessStatusRequest;
import com.example.orderfood.dto.BusinessStatusResponse;
import com.example.orderfood.dto.DashboardResponse;
import com.example.orderfood.dto.OrderResponse;
import com.example.orderfood.entity.BusinessStatus;
import com.example.orderfood.entity.FoodOrder;
import com.example.orderfood.repository.BusinessStatusRepository;
import com.example.orderfood.repository.FoodOrderRepository;
import com.example.orderfood.repository.MenuItemRepository;
import com.example.orderfood.service.BusinessService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

import java.math.BigDecimal;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.util.List;

/**
 * 门店服务实现。
 */
@Service
@RequiredArgsConstructor
public class BusinessServiceImpl implements BusinessService {

    private final BusinessStatusRepository businessStatusRepository;
    private final FoodOrderRepository foodOrderRepository;
    private final MenuItemRepository menuItemRepository;
    private final OrderAssembler orderAssembler = new OrderAssembler();

    @Override
    public BusinessStatusResponse getBusinessStatus() {
        BusinessStatus businessStatus = businessStatusRepository.findById(1L)
                .orElse(BusinessStatus.builder().id(1L).open(false).notice("暂无公告").storeName("食光小馆").build());
        return BusinessStatusResponse.builder()
                .open(businessStatus.isOpen())
                .notice(businessStatus.getNotice())
                .storeName(businessStatus.getStoreName())
                .build();
    }

    @Override
    public BusinessStatusResponse updateBusinessStatus(BusinessStatusRequest request) {
        BusinessStatus businessStatus = businessStatusRepository.findById(1L)
                .orElse(BusinessStatus.builder().id(1L).build());
        businessStatus.setOpen(request.isOpen());
        businessStatus.setStoreName(request.getStoreName());
        businessStatus.setNotice(request.getNotice());
        businessStatusRepository.save(businessStatus);
        return getBusinessStatus();
    }

    @Override
    public DashboardResponse getDashboard() {
        BusinessStatusResponse businessStatus = getBusinessStatus();
        LocalDateTime start = LocalDate.now().atStartOfDay();
        LocalDateTime end = start.plusDays(1);
        List<FoodOrder> todayOrders = foodOrderRepository.findByCreatedTimeBetween(start, end);
        BigDecimal todayRevenue = todayOrders.stream()
                .map(FoodOrder::getTotalAmount)
                .reduce(BigDecimal.ZERO, BigDecimal::add);
        List<OrderResponse> latestOrders = foodOrderRepository.findTop20ByOrderByCreatedTimeDesc()
                .stream()
                .map(orderAssembler::toResponse)
                .toList();
        return DashboardResponse.builder()
                .storeName(businessStatus.getStoreName())
                .open(businessStatus.isOpen())
                .notice(businessStatus.getNotice())
                .todayOrderCount(todayOrders.size())
                .todayRevenue(todayRevenue)
                .availableMenuCount(menuItemRepository.countByAvailableTrue())
                .latestOrders(latestOrders)
                .build();
    }
}
