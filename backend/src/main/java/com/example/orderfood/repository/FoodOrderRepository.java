package com.example.orderfood.repository;

import com.example.orderfood.entity.FoodOrder;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.data.jpa.repository.JpaRepository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;

/**
 * 订单仓储。
 */
public interface FoodOrderRepository extends JpaRepository<FoodOrder, Long> {

    @EntityGraph(attributePaths = {"items", "items.menuItem"})
    Optional<FoodOrder> findById(Long id);

    @EntityGraph(attributePaths = {"items", "items.menuItem"})
    Optional<FoodOrder> findByOrderNo(String orderNo);

    @EntityGraph(attributePaths = {"items", "items.menuItem"})
    List<FoodOrder> findTop20ByOrderByCreatedTimeDesc();

    @EntityGraph(attributePaths = {"items", "items.menuItem"})
    List<FoodOrder> findByCustomerPhoneOrderByCreatedTimeDesc(String customerPhone);

    List<FoodOrder> findByCreatedTimeBetween(LocalDateTime startTime, LocalDateTime endTime);
}
