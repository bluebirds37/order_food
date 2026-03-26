package com.example.orderfood.entity;

import jakarta.persistence.*;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.math.BigDecimal;

/**
 * 菜品实体。
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Entity
@Table(name = "menu_item")
public class MenuItem {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "category_id", nullable = false)
    private MenuCategory category;

    @Column(nullable = false, length = 64)
    private String name;

    @Column(nullable = false, precision = 10, scale = 2)
    private BigDecimal price;

    @Column(nullable = false, length = 512)
    private String description;

    @Column(nullable = false, length = 512)
    private String imageUrl;

    @Column(nullable = false)
    private boolean recommended;

    @Column(nullable = false)
    private boolean available;

    @Column(nullable = false)
    private Integer sortOrder;
}
