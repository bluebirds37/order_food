package com.example.orderfood.service.impl;

import com.example.orderfood.common.BusinessException;
import com.example.orderfood.dto.AdminLoginRequest;
import com.example.orderfood.dto.AdminLoginResponse;
import com.example.orderfood.entity.AdminUser;
import com.example.orderfood.repository.AdminUserRepository;
import com.example.orderfood.service.AdminService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

/**
 * 管理员服务实现。
 */
@Service
@RequiredArgsConstructor
public class AdminServiceImpl implements AdminService {

    private final AdminUserRepository adminUserRepository;

    @Override
    public AdminLoginResponse login(AdminLoginRequest request) {
        AdminUser adminUser = adminUserRepository.findByUsername(request.getUsername())
                .orElseThrow(() -> new BusinessException("用户名或密码错误"));
        if (!adminUser.getPassword().equals(request.getPassword())) {
            throw new BusinessException("用户名或密码错误");
        }
        return new AdminLoginResponse(adminUser.getId(), adminUser.getUsername(), adminUser.getDisplayName(), adminUser.getUsername() + "-token");
    }
}
