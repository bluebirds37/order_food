package com.example.orderfood.service.impl;

import com.example.orderfood.common.BusinessException;
import com.example.orderfood.dto.CreateOrderItemRequest;
import com.example.orderfood.dto.CreateOrderRequest;
import com.example.orderfood.dto.OrderResponse;
import com.example.orderfood.dto.OrderStatusRequest;
import com.example.orderfood.entity.FoodOrder;
import com.example.orderfood.entity.FoodOrderItem;
import com.example.orderfood.entity.MenuItem;
import com.example.orderfood.enums.OrderStatus;
import com.example.orderfood.repository.BusinessStatusRepository;
import com.example.orderfood.repository.FoodOrderRepository;
import com.example.orderfood.repository.MenuItemRepository;
import com.example.orderfood.service.OrderService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ThreadLocalRandom;

/**
 * 订单服务实现。
 */
@Service
@RequiredArgsConstructor
public class OrderServiceImpl implements OrderService {

    private final FoodOrderRepository foodOrderRepository;
    private final MenuItemRepository menuItemRepository;
    private final BusinessStatusRepository businessStatusRepository;
    private final OrderAssembler orderAssembler = new OrderAssembler();

    @Override
    @Transactional
    public OrderResponse createOrder(CreateOrderRequest request) {
        boolean open = businessStatusRepository.findById(1L).map(status -> status.isOpen()).orElse(false);
        if (!open) {
            throw new BusinessException("门店暂未营业，暂时无法下单");
        }

        FoodOrder order = new FoodOrder();
        order.setOrderNo(generateOrderNo());
        order.setCustomerName(request.getCustomerName());
        order.setCustomerPhone(request.getCustomerPhone());
        order.setDeskCode(request.getDeskCode());
        order.setStatus(OrderStatus.PAID);
        order.setRemark(request.getRemark());

        List<FoodOrderItem> orderItems = new ArrayList<>();
        BigDecimal totalAmount = BigDecimal.ZERO;
        for (CreateOrderItemRequest itemRequest : request.getItems()) {
            MenuItem menuItem = menuItemRepository.findById(itemRequest.getMenuItemId())
                    .orElseThrow(() -> new BusinessException("存在无效菜品"));
            if (!menuItem.isAvailable()) {
                throw new BusinessException(menuItem.getName() + " 当前已停售");
            }
            FoodOrderItem orderItem = FoodOrderItem.builder()
                    .order(order)
                    .menuItem(menuItem)
                    .itemName(menuItem.getName())
                    .quantity(itemRequest.getQuantity())
                    .price(menuItem.getPrice())
                    .build();
            totalAmount = totalAmount.add(menuItem.getPrice().multiply(BigDecimal.valueOf(itemRequest.getQuantity())));
            orderItems.add(orderItem);
        }
        order.setTotalAmount(totalAmount);
        order.setItems(orderItems);
        return orderAssembler.toResponse(foodOrderRepository.save(order));
    }

    @Override
    public OrderResponse getOrderDetail(Long orderId) {
        return foodOrderRepository.findById(orderId)
                .map(orderAssembler::toResponse)
                .orElseThrow(() -> new BusinessException("订单不存在"));
    }

    @Override
    public OrderResponse getOrderByOrderNo(String orderNo) {
        return foodOrderRepository.findByOrderNo(orderNo)
                .map(orderAssembler::toResponse)
                .orElseThrow(() -> new BusinessException("订单不存在"));
    }

    @Override
    public List<OrderResponse> getOrdersByPhone(String phone) {
        return foodOrderRepository.findByCustomerPhoneOrderByCreatedTimeDesc(phone)
                .stream()
                .map(orderAssembler::toResponse)
                .toList();
    }

    @Override
    public List<OrderResponse> getAllOrders() {
        return foodOrderRepository.findTop20ByOrderByCreatedTimeDesc()
                .stream()
                .map(orderAssembler::toResponse)
                .toList();
    }

    @Override
    @Transactional
    public OrderResponse updateOrderStatus(Long orderId, OrderStatusRequest request) {
        FoodOrder order = foodOrderRepository.findById(orderId)
                .orElseThrow(() -> new BusinessException("订单不存在"));
        order.setStatus(request.getStatus());
        return orderAssembler.toResponse(foodOrderRepository.save(order));
    }

    private String generateOrderNo() {
        return "OD" + LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyyMMddHHmmss"))
                + ThreadLocalRandom.current().nextInt(100, 999);
    }
}
