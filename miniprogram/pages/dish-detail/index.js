const { request } = require('../../utils/request')

Page({
  data: {
    tenantCode: '',
    dish: null,
    banners: [],
    loading: true
  },

  onLoad(options) {
    this.setData({
      tenantCode: options.tenantCode ? decodeURIComponent(options.tenantCode) : '',
      dishId: options.id ? Number(options.id) : null
    })
    this.loadDish()
  },

  async loadDish() {
    const { tenantCode, dishId } = this.data
    if (!tenantCode || !dishId) {
      this.setData({ loading: false })
      wx.showToast({ title: '菜品信息缺失', icon: 'none' })
      return
    }
    try {
      const data = await request({
        url: `/menu/overview?tenantCode=${encodeURIComponent(tenantCode)}`,
        method: 'GET'
      })
      const dish = data.menuItems.find((item) => item.id === dishId)
      if (!dish) {
        wx.showToast({ title: '菜品不存在或已下架', icon: 'none' })
        this.setData({ loading: false })
        return
      }
      const banners = (dish.detailImages || []).filter((url) => url)
      if (!banners.length && dish.imageUrl) {
        banners.push(dish.imageUrl)
      }
      this.setData({ dish, banners, loading: false })
    } catch (error) {
      this.setData({ loading: false })
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    }
  },

  addToCart() {
    const pages = getCurrentPages()
    const menuPage = pages.find((page) => page.route && page.route.indexOf('pages/menu/index') >= 0)
    if (menuPage && menuPage.addToCart) {
      menuPage.addToCart({ currentTarget: { dataset: { id: this.data.dish.id } } })
      wx.showToast({ title: '已加入购物车', icon: 'success' })
      return
    }
    wx.showToast({ title: '请从点餐页选择菜品', icon: 'none' })
  }
})
