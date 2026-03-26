package com.example.orderfood.service;

import com.example.orderfood.dto.AdminLoginRequest;
import com.example.orderfood.dto.AdminLoginResponse;

/**
 * 管理员服务。
 */
public interface AdminService {

    AdminLoginResponse login(AdminLoginRequest request);
}
