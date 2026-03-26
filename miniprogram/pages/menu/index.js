const { request } = require('../../utils/request')

Page({
  data: {
    deskCode: 'A01',
    businessStatus: {},
    categories: [],
    menuItems: [],
    cart: [],
    customerName: '',
    customerPhone: '',
    cartTypeCount: 0,
    cartTotal: '0.00'
  },

  onLoad(options) {
    const app = getApp()
    if (options.scene) {
      app.globalData.deskCode = decodeURIComponent(options.scene)
    }
    this.setData({ deskCode: app.globalData.deskCode })
    this.loadMenu()
  },

  async loadMenu() {
    try {
      const data = await request({ url: '/menu/overview', method: 'GET' })
      this.setData({
        businessStatus: data.businessStatus,
        categories: data.categories,
        menuItems: data.menuItems
      })
    } catch (error) {
      wx.showToast({ title: error.message, icon: 'none' })
    }
  },

  handleInput(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [field]: event.detail.value })
  },

  addToCart(event) {
    const { id } = event.currentTarget.dataset
    const cart = [...this.data.cart]
    const item = this.data.menuItems.find((menuItem) => menuItem.id === id)
    const target = cart.find((cartItem) => cartItem.id === id)
    if (!item) {
      return
    }
    if (target) {
      target.quantity += 1
    } else {
      cart.push({ ...item, quantity: 1 })
    }
    this.refreshCart(cart)
  },

  refreshCart(cart) {
    const total = cart.reduce((sum, item) => sum + Number(item.price) * item.quantity, 0)
    this.setData({
      cart,
      cartTypeCount: cart.length,
      cartTotal: total.toFixed(2)
    })
  },

  async submitOrder() {
    if (!this.data.businessStatus.open) {
      wx.showToast({ title: '门店当前未营业', icon: 'none' })
      return
    }
    if (!this.data.customerName || !this.data.customerPhone) {
      wx.showToast({ title: '请填写用餐人和联系电话', icon: 'none' })
      return
    }
    if (!this.data.cart.length) {
      wx.showToast({ title: '请先选择菜品', icon: 'none' })
      return
    }
    try {
      const order = await request({
        url: '/orders',
        method: 'POST',
        data: {
          customerName: this.data.customerName,
          customerPhone: this.data.customerPhone,
          deskCode: this.data.deskCode,
          items: this.data.cart.map((item) => ({ menuItemId: item.id, quantity: item.quantity }))
        }
      })
      wx.navigateTo({ url: `/pages/order-detail/index?orderId=${order.id}` })
    } catch (error) {
      wx.showToast({ title: error.message, icon: 'none' })
    }
  },

  goHistory() {
    if (!this.data.customerPhone) {
      wx.showToast({ title: '请先填写手机号', icon: 'none' })
      return
    }
    wx.navigateTo({ url: `/pages/order-history/index?phone=${this.data.customerPhone}` })
  }
})
