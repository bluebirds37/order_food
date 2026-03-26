package com.example.orderfood.service;

import com.example.orderfood.dto.MenuItemRequest;
import com.example.orderfood.dto.MenuItemResponse;
import com.example.orderfood.dto.MenuOverviewResponse;

import java.util.List;

/**
 * 菜单服务。
 */
public interface MenuService {

    MenuOverviewResponse getMenuOverview();

    List<MenuItemResponse> getAllMenuItems();

    MenuItemResponse saveMenuItem(Long id, MenuItemRequest request);

    void deleteMenuItem(Long id);
}
