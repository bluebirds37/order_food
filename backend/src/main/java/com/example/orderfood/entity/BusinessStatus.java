package com.example.orderfood.entity;

import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

/**
 * 门店营业状态。
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Entity
@Table(name = "business_status")
public class BusinessStatus {

    @Id
    private Long id;

    private boolean open;

    private String notice;

    private String storeName;
}
