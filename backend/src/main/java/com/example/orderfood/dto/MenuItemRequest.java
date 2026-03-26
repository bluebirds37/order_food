package com.example.orderfood.dto;

import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import lombok.Data;

import java.math.BigDecimal;

/**
 * 菜品编辑请求。
 */
@Data
public class MenuItemRequest {

    @NotNull(message = "分类不能为空")
    private Long categoryId;

    @NotBlank(message = "菜品名称不能为空")
    private String name;

    @NotNull(message = "价格不能为空")
    @DecimalMin(value = "0.01", message = "价格必须大于0")
    private BigDecimal price;

    @NotBlank(message = "菜品描述不能为空")
    private String description;

    @NotBlank(message = "菜品图片不能为空")
    private String imageUrl;

    private boolean recommended;

    private boolean available;

    @NotNull(message = "排序不能为空")
    private Integer sortOrder;
}
