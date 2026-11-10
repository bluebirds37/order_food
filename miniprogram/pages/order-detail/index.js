const { request } = require('../../utils/request')
const { orderStatusText } = require('../../utils/status')

const APPENDABLE_STATUS = ['PENDING_PAYMENT', 'PAID', 'PREPARING']

Page({
  data: { order: null, canAppend: false },

  onLoad(options) {
    this.orderId = options.orderId
  },

  onShow() {
    if (this.orderId) {
      this.loadOrder(this.orderId)
    }
  },

  async loadOrder(orderId) {
    try {
      const order = await request({ url: `/orders/${orderId}`, method: 'GET' })
      this.setData({
        order: { ...order, statusText: orderStatusText(order.status) },
        pickupNo: (order.orderNo || '').slice(-4),
        canAppend: APPENDABLE_STATUS.indexOf(order.status) >= 0
      })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  goAppend() {
    const { order } = this.data
    if (!order) {
      return
    }
    wx.navigateTo({
      url: `/pages/menu/index?tenantCode=${encodeURIComponent(wx.getStorageSync('tenantCode') || '')}&orderId=${order.id}&orderNo=${encodeURIComponent(order.orderNo)}`
    })
  }
})
