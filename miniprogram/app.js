App({
  globalData: {
    baseUrl: 'http://localhost:8080/api',
    deskCode: '',
    tenantCode: wx.getStorageSync('tenantCode') || '',
    tenantName: wx.getStorageSync('tenantName') || '',
    token: wx.getStorageSync('authToken') || '',
    role: wx.getStorageSync('authRole') || ''
  }
})
