App({
  globalData: {
    baseUrl: 'http://localhost:8080/api',
    deskCode: 'A01',
    adminToken: wx.getStorageSync('adminToken') || ''
  }
})
