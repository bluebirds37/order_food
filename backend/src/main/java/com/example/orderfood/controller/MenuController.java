package com.example.orderfood.controller;

import com.example.orderfood.common.ApiResponse;
import com.example.orderfood.dto.MenuItemRequest;
import com.example.orderfood.dto.MenuItemResponse;
import com.example.orderfood.dto.MenuOverviewResponse;
import com.example.orderfood.service.MenuService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

/**
 * 菜单接口。
 */
@RestController
@RequiredArgsConstructor
@RequestMapping("/api/menu")
public class MenuController {

    private final MenuService menuService;

    @GetMapping("/overview")
    public ApiResponse<MenuOverviewResponse> getOverview() {
        return ApiResponse.success(menuService.getMenuOverview());
    }

    @GetMapping("/items")
    public ApiResponse<List<MenuItemResponse>> getItems() {
        return ApiResponse.success(menuService.getAllMenuItems());
    }

    @PostMapping("/items")
    public ApiResponse<MenuItemResponse> createItem(@Valid @RequestBody MenuItemRequest request) {
        return ApiResponse.success("新增成功", menuService.saveMenuItem(null, request));
    }

    @PutMapping("/items/{id}")
    public ApiResponse<MenuItemResponse> updateItem(@PathVariable Long id, @Valid @RequestBody MenuItemRequest request) {
        return ApiResponse.success("更新成功", menuService.saveMenuItem(id, request));
    }

    @DeleteMapping("/items/{id}")
    public ApiResponse<Void> deleteItem(@PathVariable Long id) {
        menuService.deleteMenuItem(id);
        return ApiResponse.success("删除成功", null);
    }
}
