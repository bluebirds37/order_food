const { request } = require('../../utils/request')

Page({
  data: {
    status: {
      storeName: '',
      notice: '',
      open: false
    }
  },

  onShow() {
    if (!wx.getStorageSync('adminToken')) {
      wx.redirectTo({ url: '/pages/admin-login/index' })
      return
    }
    this.loadData()
  },

  async loadData() {
    try {
      const status = await request({ url: '/business/status', method: 'GET' })
      this.setData({ status })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  handleInput(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [`status.${field}`]: event.detail.value })
  },

  handleSwitch(event) {
    this.setData({ 'status.open': event.detail.value })
  },

  async saveStatus() {
    try {
      await request({ url: '/business/status', method: 'PUT', data: this.data.status })
      wx.showToast({ title: '保存成功', icon: 'success' })
    } catch (error) {
      wx.showToast({ title: error.message || '保存失败', icon: 'none' })
    }
  },

  backDashboard() {
    wx.navigateTo({ url: '/pages/admin/index' })
  }
})
