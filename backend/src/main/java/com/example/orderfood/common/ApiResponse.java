package com.example.orderfood.common;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

/**
 * 统一响应体。
 *
 * @param <T> 数据类型
 */
@Data
@NoArgsConstructor
@AllArgsConstructor
public class ApiResponse<T> {

    private boolean success;

    private String message;

    private T data;

    public static <T> ApiResponse<T> success(T data) {
        return new ApiResponse<>(true, "操作成功", data);
    }

    public static <T> ApiResponse<T> success(String message, T data) {
        return new ApiResponse<>(true, message, data);
    }

    public static <T> ApiResponse<T> fail(String message) {
        return new ApiResponse<>(false, message, null);
    }
}
