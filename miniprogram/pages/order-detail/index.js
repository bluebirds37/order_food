const { request } = require('../../utils/request')

Page({
  data: { order: null },

  onLoad(options) {
    this.loadOrder(options.orderId)
  },

  async loadOrder(orderId) {
    const order = await request({ url: `/orders/${orderId}`, method: 'GET' })
    this.setData({ order })
  }
})
