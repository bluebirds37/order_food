package com.example.orderfood.service.impl;

import com.example.orderfood.common.BusinessException;
import com.example.orderfood.dto.*;
import com.example.orderfood.entity.MenuCategory;
import com.example.orderfood.entity.MenuItem;
import com.example.orderfood.repository.BusinessStatusRepository;
import com.example.orderfood.repository.MenuCategoryRepository;
import com.example.orderfood.repository.MenuItemRepository;
import com.example.orderfood.service.MenuService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

/**
 * 菜单服务实现。
 */
@Service
@RequiredArgsConstructor
public class MenuServiceImpl implements MenuService {

    private final MenuCategoryRepository menuCategoryRepository;
    private final MenuItemRepository menuItemRepository;
    private final BusinessStatusRepository businessStatusRepository;

    @Override
    public MenuOverviewResponse getMenuOverview() {
        List<MenuCategoryResponse> categories = menuCategoryRepository.findAllByOrderBySortOrderAscIdAsc()
                .stream()
                .map(category -> MenuCategoryResponse.builder()
                        .id(category.getId())
                        .name(category.getName())
                        .sortOrder(category.getSortOrder())
                        .build())
                .toList();
        List<MenuItemResponse> items = menuItemRepository.findAllByOrderBySortOrderAscIdAsc()
                .stream()
                .map(this::toResponse)
                .toList();
        BusinessStatusResponse businessStatus = businessStatusRepository.findById(1L)
                .map(status -> BusinessStatusResponse.builder()
                        .open(status.isOpen())
                        .notice(status.getNotice())
                        .storeName(status.getStoreName())
                        .build())
                .orElse(BusinessStatusResponse.builder().open(false).notice("暂无公告").storeName("食光小馆").build());
        return MenuOverviewResponse.builder()
                .businessStatus(businessStatus)
                .categories(categories)
                .menuItems(items)
                .build();
    }

    @Override
    public List<MenuItemResponse> getAllMenuItems() {
        return menuItemRepository.findAllByOrderBySortOrderAscIdAsc()
                .stream()
                .map(this::toResponse)
                .toList();
    }

    @Override
    @Transactional
    public MenuItemResponse saveMenuItem(Long id, MenuItemRequest request) {
        MenuCategory category = menuCategoryRepository.findById(request.getCategoryId())
                .orElseThrow(() -> new BusinessException("菜品分类不存在"));
        MenuItem menuItem = id == null ? new MenuItem() : menuItemRepository.findById(id)
                .orElseThrow(() -> new BusinessException("菜品不存在"));
        menuItem.setCategory(category);
        menuItem.setName(request.getName());
        menuItem.setPrice(request.getPrice());
        menuItem.setDescription(request.getDescription());
        menuItem.setImageUrl(request.getImageUrl());
        menuItem.setRecommended(request.isRecommended());
        menuItem.setAvailable(request.isAvailable());
        menuItem.setSortOrder(request.getSortOrder());
        return toResponse(menuItemRepository.save(menuItem));
    }

    @Override
    @Transactional
    public void deleteMenuItem(Long id) {
        menuItemRepository.deleteById(id);
    }

    private MenuItemResponse toResponse(MenuItem menuItem) {
        return MenuItemResponse.builder()
                .id(menuItem.getId())
                .categoryId(menuItem.getCategory().getId())
                .categoryName(menuItem.getCategory().getName())
                .name(menuItem.getName())
                .price(menuItem.getPrice())
                .description(menuItem.getDescription())
                .imageUrl(menuItem.getImageUrl())
                .recommended(menuItem.isRecommended())
                .available(menuItem.isAvailable())
                .sortOrder(menuItem.getSortOrder())
                .build();
    }
}
