package com.example.orderfood.dto;

import jakarta.validation.constraints.NotBlank;
import lombok.Data;

/**
 * 营业状态更新请求。
 */
@Data
public class BusinessStatusRequest {

    private boolean open;

    @NotBlank(message = "门店名称不能为空")
    private String storeName;

    @NotBlank(message = "营业公告不能为空")
    private String notice;
}
