package com.example.orderfood.dto;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import lombok.Data;

import java.util.List;

/**
 * 创建订单请求。
 */
@Data
public class CreateOrderRequest {

    @NotBlank(message = "用餐人不能为空")
    private String customerName;

    @NotBlank(message = "联系电话不能为空")
    private String customerPhone;

    @NotBlank(message = "桌号不能为空")
    private String deskCode;

    private String remark;

    @Valid
    @NotEmpty(message = "请至少选择一道菜品")
    private List<CreateOrderItemRequest> items;
}
