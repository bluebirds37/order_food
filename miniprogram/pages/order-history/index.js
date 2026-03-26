const { request } = require('../../utils/request')

Page({
  data: { orders: [] },

  onLoad(options) {
    this.loadOrders(options.phone)
  },

  async loadOrders(phone) {
    const orders = await request({ url: `/orders/history?phone=${phone}`, method: 'GET' })
    this.setData({ orders })
  }
})
