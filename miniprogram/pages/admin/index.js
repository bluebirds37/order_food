const { request } = require('../../utils/request')

Page({
  data: {
    dashboard: {
      storeName: '食光小馆',
      open: false,
      notice: '',
      todayOrderCount: 0,
      todayRevenue: 0,
      availableMenuCount: 0,
      latestOrders: []
    }
  },

  onShow() {
    if (!wx.getStorageSync('adminToken')) {
      wx.redirectTo({ url: '/pages/admin-login/index' })
      return
    }
    this.loadDashboard()
  },

  async loadDashboard() {
    try {
      const dashboard = await request({ url: '/admin/dashboard', method: 'GET' })
      this.setData({ dashboard })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  navigateToPage(event) {
    const { url } = event.currentTarget.dataset
    wx.navigateTo({ url })
  },

  logout() {
    wx.removeStorageSync('adminToken')
    wx.redirectTo({ url: '/pages/admin-login/index' })
  }
})
