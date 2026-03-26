package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

import java.math.BigDecimal;

/**
 * 菜品响应。
 */
@Data
@Builder
public class MenuItemResponse {

    private Long id;

    private Long categoryId;

    private String categoryName;

    private String name;

    private BigDecimal price;

    private String description;

    private String imageUrl;

    private boolean recommended;

    private boolean available;

    private Integer sortOrder;
}
