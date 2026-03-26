package com.example.orderfood.dto;

import lombok.AllArgsConstructor;
import lombok.Data;

/**
 * 管理员登录响应。
 */
@Data
@AllArgsConstructor
public class AdminLoginResponse {

    private Long adminId;

    private String username;

    private String displayName;

    private String token;
}
