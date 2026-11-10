const { request } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')
const { withStatusText } = require('../../utils/status')

Page({
  data: {
    tenantCode: '',
    tenantName: '',
    adminName: '',
    role: '',
    dashboard: {
      storeName: '',
      open: false,
      notice: '',
      todayOrderCount: 0,
      todayRevenue: 0,
      availableMenuCount: 0,
      latestOrders: []
    }
  },

  onShow() {
    const session = requireAdmin()
    if (!session) {
      return
    }
    const app = getApp()
    this.setData({
      tenantCode: session.tenantCode,
      tenantName: app.globalData.tenantName || wx.getStorageSync('tenantName') || session.tenantCode,
      adminName: wx.getStorageSync('adminName') || '',
      role: session.role
    })
    this.loadDashboard(session.tenantCode)
  },

  async loadDashboard(tenantCode) {
    try {
      const dashboard = await request({
        url: `/admin/dashboard${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
      this.setData({
        dashboard: {
          ...dashboard,
          latestOrders: withStatusText(dashboard.latestOrders)
        }
      })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  navigateToPage(event) {
    const { url } = event.currentTarget.dataset
    wx.navigateTo({ url })
  },

  logout() {
    wx.removeStorageSync('authToken')
    wx.removeStorageSync('authRole')
    wx.removeStorageSync('adminName')
    const app = getApp()
    app.globalData.token = ''
    app.globalData.role = ''
    wx.redirectTo({ url: '/pages/admin-login/index' })
  }
})
