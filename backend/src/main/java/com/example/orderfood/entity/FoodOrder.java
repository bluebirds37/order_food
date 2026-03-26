package com.example.orderfood.entity;

import com.example.orderfood.enums.OrderStatus;
import jakarta.persistence.*;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;

/**
 * 订单主表。
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Entity
@Table(name = "food_order")
public class FoodOrder {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true, length = 32)
    private String orderNo;

    @Column(nullable = false, length = 32)
    private String customerName;

    @Column(nullable = false, length = 32)
    private String customerPhone;

    @Column(nullable = false, length = 32)
    private String deskCode;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 32)
    private OrderStatus status;

    @Column(nullable = false, precision = 10, scale = 2)
    private BigDecimal totalAmount;

    @Column(length = 255)
    private String remark;

    @OneToMany(mappedBy = "order", cascade = CascadeType.ALL, orphanRemoval = true)
    @Builder.Default
    private List<FoodOrderItem> items = new ArrayList<>();

    @CreationTimestamp
    private LocalDateTime createdTime;

    @UpdateTimestamp
    private LocalDateTime updatedTime;
}
