const { request } = require('../../utils/request')

const STATUS_OPTIONS = ['PAID', 'PREPARING', 'COMPLETED', 'CANCELLED']

Page({
  data: {
    loading: false,
    orders: [],
    statusOptions: STATUS_OPTIONS
  },

  onShow() {
    if (!wx.getStorageSync('adminToken')) {
      wx.redirectTo({ url: '/pages/admin-login/index' })
      return
    }
    this.loadData()
  },

  async loadData() {
    this.setData({ loading: true })
    try {
      const orders = await request({ url: '/orders', method: 'GET' })
      this.setData({ orders })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  async updateStatus(event) {
    const { id, status } = event.currentTarget.dataset
    try {
      await request({
        url: `/orders/${id}/status`,
        method: 'PUT',
        data: { status }
      })
      wx.showToast({ title: '状态已更新', icon: 'success' })
      this.loadData()
    } catch (error) {
      wx.showToast({ title: error.message || '更新失败', icon: 'none' })
    }
  }
})
