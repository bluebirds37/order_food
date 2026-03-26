package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

import java.util.List;

/**
 * 菜单总览响应。
 */
@Data
@Builder
public class MenuOverviewResponse {

    private BusinessStatusResponse businessStatus;

    private List<MenuCategoryResponse> categories;

    private List<MenuItemResponse> menuItems;
}
