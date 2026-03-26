package com.example.orderfood.config;

import com.example.orderfood.entity.AdminUser;
import com.example.orderfood.entity.BusinessStatus;
import com.example.orderfood.entity.MenuCategory;
import com.example.orderfood.entity.MenuItem;
import com.example.orderfood.repository.AdminUserRepository;
import com.example.orderfood.repository.BusinessStatusRepository;
import com.example.orderfood.repository.MenuCategoryRepository;
import com.example.orderfood.repository.MenuItemRepository;
import org.springframework.boot.CommandLineRunner;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.math.BigDecimal;
import java.util.List;

/**
 * 启动初始化数据。
 */
@Configuration
public class DataInitializer {

    @Bean
    public CommandLineRunner initData(AdminUserRepository adminUserRepository,
                                      BusinessStatusRepository businessStatusRepository,
                                      MenuCategoryRepository menuCategoryRepository,
                                      MenuItemRepository menuItemRepository) {
        return args -> {
            if (adminUserRepository.count() == 0) {
                adminUserRepository.save(AdminUser.builder()
                        .username("admin")
                        .password("admin")
                        .displayName("门店老板")
                        .build());
            }

            if (businessStatusRepository.count() == 0) {
                businessStatusRepository.save(BusinessStatus.builder()
                        .id(1L)
                        .open(true)
                        .notice("欢迎光临，现点现做")
                        .storeName("食光小馆")
                        .build());
            }

            if (menuCategoryRepository.count() == 0) {
                MenuCategory hot = menuCategoryRepository.save(MenuCategory.builder().name("招牌热销").sortOrder(1).build());
                MenuCategory drink = menuCategoryRepository.save(MenuCategory.builder().name("饮品小食").sortOrder(2).build());
                menuItemRepository.saveAll(List.of(
                        MenuItem.builder()
                                .category(hot)
                                .name("黑椒牛肉饭")
                                .description("嫩煎牛肉搭配流心温泉蛋，口感浓郁。")
                                .price(new BigDecimal("28.00"))
                                .imageUrl("https://images.unsplash.com/photo-1512058564366-18510be2db19?auto=format&fit=crop&w=800&q=80")
                                .recommended(true)
                                .available(true)
                                .sortOrder(1)
                                .build(),
                        MenuItem.builder()
                                .category(hot)
                                .name("香酥鸡腿饭")
                                .description("外酥里嫩，搭配时蔬与秘制酱汁。")
                                .price(new BigDecimal("24.00"))
                                .imageUrl("https://images.unsplash.com/photo-1544025162-d76694265947?auto=format&fit=crop&w=800&q=80")
                                .recommended(true)
                                .available(true)
                                .sortOrder(2)
                                .build(),
                        MenuItem.builder()
                                .category(drink)
                                .name("青柠气泡水")
                                .description("清爽解腻，适合搭配主食。")
                                .price(new BigDecimal("12.00"))
                                .imageUrl("https://images.unsplash.com/photo-1513558161293-cdaf765ed2fd?auto=format&fit=crop&w=800&q=80")
                                .recommended(false)
                                .available(true)
                                .sortOrder(1)
                                .build()
                ));
            }
        };
    }
}
