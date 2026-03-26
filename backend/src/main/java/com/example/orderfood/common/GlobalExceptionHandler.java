package com.example.orderfood.common;

import jakarta.validation.ConstraintViolationException;
import org.springframework.http.converter.HttpMessageNotReadableException;
import org.springframework.web.bind.MethodArgumentNotValidException;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;

/**
 * 全局异常处理器。
 */
@RestControllerAdvice
public class GlobalExceptionHandler {

    @ExceptionHandler(BusinessException.class)
    public ApiResponse<Void> handleBusinessException(BusinessException exception) {
        return ApiResponse.fail(exception.getMessage());
    }

    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ApiResponse<Void> handleMethodArgumentNotValidException(MethodArgumentNotValidException exception) {
        String message = exception.getBindingResult().getFieldError() == null
                ? "请求参数不合法"
                : exception.getBindingResult().getFieldError().getDefaultMessage();
        return ApiResponse.fail(message);
    }

    @ExceptionHandler({ConstraintViolationException.class, HttpMessageNotReadableException.class})
    public ApiResponse<Void> handleBadRequestException(Exception exception) {
        return ApiResponse.fail("请求参数格式错误");
    }

    @ExceptionHandler(Exception.class)
    public ApiResponse<Void> handleException(Exception exception) {
        return ApiResponse.fail("系统繁忙，请稍后重试");
    }
}
