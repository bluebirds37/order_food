const { request } = require('../../utils/request')

Page({
  data: {
    username: 'admin',
    password: 'admin',
    loading: false
  },

  onShow() {
    if (wx.getStorageSync('adminToken')) {
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
      const data = await request({
        url: '/admin/login',
        method: 'POST',
        data: {
          username: this.data.username,
          password: this.data.password
        }
      })
      wx.setStorageSync('adminToken', data.token)
      wx.showToast({ title: '登录成功', icon: 'success' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/admin/index' })
      }, 300)
    } catch (error) {
      wx.showToast({ title: error.message || '登录失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  }
})
