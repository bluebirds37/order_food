package com.example.orderfood.repository;

import com.example.orderfood.entity.MenuCategory;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;

/**
 * 菜品分类仓储。
 */
public interface MenuCategoryRepository extends JpaRepository<MenuCategory, Long> {

    List<MenuCategory> findAllByOrderBySortOrderAscIdAsc();
}
