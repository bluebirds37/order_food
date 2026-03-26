package com.example.orderfood.repository;

import com.example.orderfood.entity.MenuItem;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;

/**
 * 菜品仓储。
 */
public interface MenuItemRepository extends JpaRepository<MenuItem, Long> {

    @EntityGraph(attributePaths = {"category"})
    List<MenuItem> findAllByOrderBySortOrderAscIdAsc();

    long countByAvailableTrue();
}