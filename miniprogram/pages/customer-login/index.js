const { request } = require('../../utils/request')

Page({
  data: {
    tenantCode: '',
    tenantName: '',
    mode: 'login',
    phone: wx.getStorageSync('customerPhone') || '',
    password: '',
    nickname: '',
    loading: false
  },

  onLoad(options) {
    const app = getApp()
    const tenantCode = options.tenantCode || app.globalData.tenantCode || wx.getStorageSync('tenantCode')
    const tenantName = decodeURIComponent(options.tenantName || '') || app.globalData.tenantName || wx.getStorageSync('tenantName')
    if (!tenantCode) {
      wx.showToast({ title: '请先扫码进入门店', icon: 'none' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/entry/index' })
      }, 300)
      return
    }
    this.setData({ tenantCode, tenantName })
  },

  switchMode() {
    this.setData({ mode: this.data.mode === 'login' ? 'register' : 'login' })
  },

  handleInput(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [field]: event.detail.value })
  },

  async submit() {
    if (this.data.loading) {
      return
    }
    this.setData({ loading: true })
    try {
      const payload = {
        tenantCode: this.data.tenantCode,
        phone: this.data.phone,
        password: this.data.password
      }
      let data
      if (this.data.mode === 'register') {
        data = await request({
          url: '/auth/customer/register',
          method: 'POST',
          data: Object.assign({}, payload, { nickname: this.data.nickname })
        })
      } else {
        data = await request({
          url: '/auth/customer/login',
          method: 'POST',
          data: payload
        })
      }
      const app = getApp()
      app.globalData.token = data.token
      app.globalData.role = 'CUSTOMER'
      app.globalData.tenantCode = data.tenantCode
      app.globalData.tenantName = this.data.tenantName
      wx.setStorageSync('authToken', data.token)
      wx.setStorageSync('authRole', 'CUSTOMER')
      wx.setStorageSync('tenantCode', data.tenantCode)
      wx.setStorageSync('customerName', data.nickname)
      wx.setStorageSync('customerPhone', data.phone)
      wx.setStorageSync('customerNickname', data.nickname)
      wx.showToast({ title: '登录成功', icon: 'success' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/menu/index' })
      }, 300)
    } catch (error) {
      wx.showToast({ title: error.message || '操作失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  }
})
