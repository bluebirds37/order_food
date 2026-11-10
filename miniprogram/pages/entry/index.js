const { request } = require('../../utils/request')
const { parseTenantCode } = require('../../utils/tenant')

Page({
  data: {
    codeInput: '',
    loading: false,
    tenantName: wx.getStorageSync('tenantName') || ''
  },

  onLoad(options) {
    const raw =
      options.tenantCode ||
      options.code ||
      (options.q ? decodeURIComponent(options.q) : '') ||
      options.scene ||
      ''
    if (raw) {
      this.enterTenant(decodeURIComponent(raw))
    }
  },

  handleInput(event) {
    this.setData({ codeInput: event.detail.value })
  },

  scanCode() {
    wx.scanCode({
      success: (res) => {
        this.enterTenant(res.result)
      },
      fail: () => {
        wx.showToast({ title: '未识别到二维码', icon: 'none' })
      }
    })
  },

  enterManual() {
    this.enterTenant(this.data.codeInput)
  },

  goAdminLogin() {
    const app = getApp()
    const code = app.globalData.tenantCode || wx.getStorageSync('tenantCode')
    if (!code) {
      wx.showToast({ title: '请先扫码或输入门店编码', icon: 'none' })
      return
    }
    wx.navigateTo({
      url: `/pages/admin-login/index?tenantCode=${encodeURIComponent(code)}`
    })
  },

  async enterTenant(raw) {
    const code = parseTenantCode(raw)
    if (!code) {
      wx.showToast({ title: '二维码缺少门店标识', icon: 'none' })
      return
    }
    if (this.data.loading) {
      return
    }
    this.setData({ loading: true })
    try {
      const tenant = await request({
        url: `/tenants/${code}/entry`,
        method: 'GET'
      })
      const app = getApp()
      app.globalData.tenantCode = tenant.code
      app.globalData.tenantName = tenant.name
      wx.setStorageSync('tenantCode', tenant.code)
      wx.setStorageSync('tenantName', tenant.name)
      wx.showToast({ title: `进入 ${tenant.name}`, icon: 'success' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/menu/index' })
      }, 300)
    } catch (error) {
      wx.showToast({ title: error.message || '门店不存在', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  }
})
