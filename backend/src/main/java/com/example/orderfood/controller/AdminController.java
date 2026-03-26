package com.example.orderfood.controller;

import com.example.orderfood.common.ApiResponse;
import com.example.orderfood.dto.AdminLoginRequest;
import com.example.orderfood.dto.AdminLoginResponse;
import com.example.orderfood.dto.DashboardResponse;
import com.example.orderfood.service.AdminService;
import com.example.orderfood.service.BusinessService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * 管理员接口。
 */
@RestController
@RequiredArgsConstructor
@RequestMapping("/api/admin")
public class AdminController {

    private final AdminService adminService;
    private final BusinessService businessService;

    @PostMapping("/login")
    public ApiResponse<AdminLoginResponse> login(@Valid @RequestBody AdminLoginRequest request) {
        return ApiResponse.success("登录成功", adminService.login(request));
    }

    @GetMapping("/dashboard")
    public ApiResponse<DashboardResponse> getDashboard() {
        return ApiResponse.success(businessService.getDashboard());
    }
}
