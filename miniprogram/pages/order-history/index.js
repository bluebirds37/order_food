const { request } = require('../../utils/request')
const { withStatusText } = require('../../utils/status')

Page({
  data: { orders: [] },

  onShow() {
    const token = wx.getStorageSync('authToken')
    const role = wx.getStorageSync('authRole')
    if (!token || role !== 'CUSTOMER') {
      wx.showToast({ title: '请先登录顾客账号', icon: 'none' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/customer-login/index' })
      }, 300)
      return
    }
    this.loadOrders()
  },

  async loadOrders() {
    try {
      const orders = await request({ url: '/orders/history', method: 'GET' })
      this.setData({ orders: withStatusText(orders) })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  openDetail(event) {
    const { id } = event.currentTarget.dataset
    wx.navigateTo({ url: `/pages/order-detail/index?orderId=${id}` })
  }
})
