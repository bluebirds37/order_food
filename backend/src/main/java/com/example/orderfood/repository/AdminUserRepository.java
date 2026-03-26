package com.example.orderfood.repository;

import com.example.orderfood.entity.AdminUser;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;

/**
 * 管理员仓储。
 */
public interface AdminUserRepository extends JpaRepository<AdminUser, Long> {

    Optional<AdminUser> findByUsername(String username);
}
