const { request } = require('../../utils/request')

Page({
  data: {
    tenantCode: '',
    tenantName: '',
    username: '',
    password: '',
    loading: false
  },

  onLoad(options) {
    const app = getApp()
    const tenantCode = options.tenantCode || app.globalData.tenantCode || wx.getStorageSync('tenantCode')
    const tenantName = decodeURIComponent(options.tenantName || '') || app.globalData.tenantName || wx.getStorageSync('tenantName')
    // 超级管理员可不扫码直接登录；门店管理员仍需门店上下文
    this.setData({ tenantCode: tenantCode || '', tenantName: tenantName || '' })
  },

  onShow() {
    const token = wx.getStorageSync('authToken')
    const role = wx.getStorageSync('authRole')
    if (token && (role === 'TENANT_ADMIN' || role === 'SUPER_ADMIN')) {
      wx.redirectTo({ url: '/pages/admin/index' })
    }
  },

  handleInput(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [field]: event.detail.value })
  },

  async login() {
    if (this.data.loading) {
      return
    }
    this.setData({ loading: true })
    try {
      const loginPayload = {
        username: this.data.username,
        password: this.data.password
      }
      if (this.data.tenantCode) {
        loginPayload.tenantCode = this.data.tenantCode
      }
      const data = await request({
        url: '/admin/login',
        method: 'POST',
        data: loginPayload
      })
      const app = getApp()
      app.globalData.token = data.token
      app.globalData.role = data.role
      if (data.tenantCode) {
        app.globalData.tenantCode = data.tenantCode
        app.globalData.tenantName = data.tenantName
        wx.setStorageSync('tenantCode', data.tenantCode)
        wx.setStorageSync('tenantName', data.tenantName)
      }
      wx.setStorageSync('authToken', data.token)
      wx.setStorageSync('authRole', data.role)
      wx.setStorageSync('adminName', data.displayName)
      wx.showToast({ title: '登录成功', icon: 'success' })
      // 超级管理员：先进入门店选择页；门店管理员：直接进入管理首页
      const target = data.role === 'SUPER_ADMIN' && !data.tenantCode
        ? '/pages/admin-tenants/index'
        : '/pages/admin/index'
      setTimeout(() => {
        wx.redirectTo({ url: target })
      }, 300)
    } catch (error) {
      wx.showToast({ title: error.message || '登录失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  }
})
