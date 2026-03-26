package com.example.orderfood.service;

import com.example.orderfood.dto.BusinessStatusRequest;
import com.example.orderfood.dto.BusinessStatusResponse;
import com.example.orderfood.dto.DashboardResponse;

/**
 * 门店服务。
 */
public interface BusinessService {

    BusinessStatusResponse getBusinessStatus();

    BusinessStatusResponse updateBusinessStatus(BusinessStatusRequest request);

    DashboardResponse getDashboard();
}
