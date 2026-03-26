package com.example.orderfood.dto;

import lombok.Builder;
import lombok.Data;

/**
 * 营业状态响应。
 */
@Data
@Builder
public class BusinessStatusResponse {

    private boolean open;

    private String notice;

    private String storeName;
}
