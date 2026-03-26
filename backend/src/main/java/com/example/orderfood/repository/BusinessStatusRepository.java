package com.example.orderfood.repository;

import com.example.orderfood.entity.BusinessStatus;
import org.springframework.data.jpa.repository.JpaRepository;

/**
 * 营业状态仓储。
 */
public interface BusinessStatusRepository extends JpaRepository<BusinessStatus, Long> {
}
