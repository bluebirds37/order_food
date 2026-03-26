package com.example.orderfood.controller;

import com.example.orderfood.common.ApiResponse;
import com.example.orderfood.dto.BusinessStatusRequest;
import com.example.orderfood.dto.BusinessStatusResponse;
import com.example.orderfood.service.BusinessService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * 门店接口。
 */
@RestController
@RequiredArgsConstructor
@RequestMapping("/api/business")
public class BusinessController {

    private final BusinessService businessService;

    @GetMapping("/status")
    public ApiResponse<BusinessStatusResponse> getStatus() {
        return ApiResponse.success(businessService.getBusinessStatus());
    }

    @PutMapping("/status")
    public ApiResponse<BusinessStatusResponse> updateStatus(@Valid @RequestBody BusinessStatusRequest request) {
        return ApiResponse.success("更新成功", businessService.updateBusinessStatus(request));
    }
}
