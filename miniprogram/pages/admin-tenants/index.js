const { request } = require('../../utils/request')

Page({
  data: {
    loading: false,
    tenants: []
  },

  onShow() {
    const token = wx.getStorageSync('authToken')
    const role = wx.getStorageSync('authRole')
    if (!token || role !== 'SUPER_ADMIN') {
      wx.showToast({ title: '请使用超级管理员登录', icon: 'none' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/admin-login/index' })
      }, 300)
      return
    }
    this.loadTenants()
  },

  onPullDownRefresh() {
    this.loadTenants().finally(() => wx.stopPullDownRefresh())
  },

  async loadTenants() {
    this.setData({ loading: true })
    try {
      const tenants = await request({ url: '/super/tenants', method: 'GET' })
      this.setData({ tenants })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  switchTenant(event) {
    const { code, name } = event.currentTarget.dataset
    const app = getApp()
    app.globalData.tenantCode = code
    app.globalData.tenantName = name
    wx.setStorageSync('tenantCode', code)
    wx.setStorageSync('tenantName', name)
    wx.showToast({ title: `已切换到 ${name}`, icon: 'success' })
    setTimeout(() => {
      wx.redirectTo({ url: '/pages/admin/index' })
    }, 300)
  }
})
