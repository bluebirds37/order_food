package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

/**
 * 菜品分类响应。
 */
@Data
@Builder
public class MenuCategoryResponse {

    private Long id;

    private String name;

    private Integer sortOrder;
}
