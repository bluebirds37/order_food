const { request } = require('../../utils/request')
const { parseTenantCode } = require('../../utils/tenant')
const { silentWxLogin, isPlaceholderPhone } = require('../../utils/auth')

Page({
  data: {
    tenantCode: '',
    tenantName: '',
    deskCode: '',
    appendOrderId: null,
    appendOrderNo: '',
    businessStatus: {},
    categories: [],
    menuItems: [],
    cart: [],
    customerName: '',
    customerPhone: '',
    customerNickname: '',
    loggedIn: false,
    preview: false,
    cartTypeCount: 0,
    cartTotal: '0.00',
    wxLoggingIn: false,
    needBindPhone: false,
    searchText: '',
    searching: false,
    menuGroups: [],
    filteredItems: [],
    activeCategoryId: null,
    dishPopup: false,
    popupItem: null,
    popupOptions: [],
    popupSelectedFlavor: '',
    popupRemark: '',
    popupQuantity: 1,
    desks: [],
    deskLoading: false,
    deskPopup: false
  },

  onLoad(options) {
    const app = getApp()
    const raw =
      options.tenantCode ||
      options.code ||
      (options.q ? decodeURIComponent(options.q) : '') ||
      options.scene ||
      ''
    const code =
      parseTenantCode(decodeURIComponent(raw || '')) ||
      app.globalData.tenantCode ||
      wx.getStorageSync('tenantCode')

    if (!code) {
      wx.showToast({ title: '缺少门店标识，请先扫码', icon: 'none' })
      setTimeout(() => {
        wx.redirectTo({ url: '/pages/entry/index' })
      }, 300)
      return
    }

    app.globalData.tenantCode = code
    const deskCode =
      options.deskCode || app.globalData.deskCode || wx.getStorageSync(`deskCode_${code}`) || ''
    if (deskCode) {
      app.globalData.deskCode = deskCode
    }
    this.setData({
      tenantCode: code,
      tenantName: app.globalData.tenantName || wx.getStorageSync('tenantName') || code,
      deskCode,
      preview: options.preview === '1',
      appendOrderId: options.orderId ? Number(options.orderId) : null,
      appendOrderNo: options.orderNo ? decodeURIComponent(options.orderNo) : ''
    })
  },

  onShow() {
    if (!this.data.tenantCode) {
      return
    }
    this.refreshCustomer()
    this.loadMenu()
    if (!this.data.preview) {
      this.ensureWxLogin()
    }
  },

  refreshCustomer() {
    const role = wx.getStorageSync('authRole')
    if (role !== 'CUSTOMER' || !wx.getStorageSync('authToken')) {
      this.setData({ loggedIn: false, customerNickname: '', customerPhone: '', needBindPhone: false })
      return
    }
    const phone = wx.getStorageSync('customerPhone') || ''
    this.setData({
      loggedIn: true,
      customerNickname: wx.getStorageSync('customerNickname') || '',
      customerPhone: phone,
      needBindPhone: isPlaceholderPhone(phone)
    })
  },

  ensureWxLogin() {
    if (this.data.loggedIn || this.data.wxLoggingIn || this.data.preview) {
      return
    }
    this.setData({ wxLoggingIn: true })
    silentWxLogin(this.data.tenantCode)
      .then(() => {
        this.setData({ wxLoggingIn: false })
        this.refreshCustomer()
      })
      .catch((error) => {
        this.setData({ wxLoggingIn: false })
        wx.showToast({ title: error.message || '微信登录失败，可点击按钮重试', icon: 'none' })
      })
  },

  // 微信一键绑定手机号：优先用 getPhoneNumber 的 code，失败或不可用时降级为手动输入
  onGetPhoneNumber(event) {
    const detail = event.detail || {}
    if (!detail.code) {
      this.manualBindPhone()
      return
    }
    request({
      url: '/auth/customer/bind-phone',
      method: 'POST',
      data: { phoneCode: detail.code }
    })
      .then((data) => {
        wx.setStorageSync('customerPhone', data.phone)
        this.refreshCustomer()
        wx.showToast({ title: '手机号绑定成功', icon: 'success' })
      })
      .catch(() => {
        // 一键绑定失败（如未配置微信密钥），降级手动输入
        this.manualBindPhone()
      })
  },

  manualBindPhone() {
    wx.showModal({
      title: '绑定手机号',
      editable: true,
      placeholderText: '请输入 11 位手机号',
      success: (result) => {
        if (!result.confirm) {
          return
        }
        const phone = (result.content || '').trim()
        if (!/^\d{11}$/.test(phone)) {
          wx.showToast({ title: '请输入 11 位手机号', icon: 'none' })
          return
        }
        request({
          url: '/auth/customer/bind-phone',
          method: 'POST',
          data: { phone }
        })
          .then((data) => {
            wx.setStorageSync('customerPhone', data.phone)
            this.refreshCustomer()
            wx.showToast({ title: '手机号绑定成功', icon: 'success' })
          })
          .catch((error) => {
            wx.showToast({ title: error.message || '绑定失败', icon: 'none' })
          })
      }
    })
  },

  async loadMenu() {
    try {
      const data = await request({
        url: `/menu/overview?tenantCode=${encodeURIComponent(this.data.tenantCode)}`,
        method: 'GET'
      })
      this.setData({
        tenantName: data.tenant.name || this.data.tenantName,
        businessStatus: data.businessStatus,
        categories: data.categories,
        menuItems: data.menuItems
      })
      this.buildMenu(data.menuItems)
    } catch (error) {
      wx.showToast({ title: error.message, icon: 'none' })
    }
  },

  // 按分类分组展示；搜索时平铺展示匹配结果
  buildMenu(menuItems) {
    const groups = this.data.categories
      .map((category) => ({
        id: category.id,
        name: category.name,
        items: menuItems.filter((item) => item.categoryId === category.id)
      }))
      .filter((group) => group.items.length)
    const uncategorized = menuItems.filter(
      (item) => !this.data.categories.some((category) => category.id === item.categoryId)
    )
    if (uncategorized.length) {
      groups.push({ id: -1, name: '其他', items: uncategorized })
    }
    this.setData({
      menuGroups: groups,
      activeCategoryId: groups.length ? groups[0].id : null
    })
    this.applySearch()
  },

  onSearchInput(event) {
    this.setData({ searchText: event.detail.value })
    this.applySearch()
  },

  clearSearch() {
    this.setData({ searchText: '' })
    this.applySearch()
  },

  applySearch() {
    const keyword = this.data.searchText.trim().toLowerCase()
    if (!keyword) {
      this.setData({ searching: false, filteredItems: [] })
      return
    }
    const filteredItems = this.data.menuItems.filter((item) => {
      const name = (item.name || '').toLowerCase()
      const desc = (item.description || '').toLowerCase()
      const category = (item.categoryName || '').toLowerCase()
      return name.includes(keyword) || desc.includes(keyword) || category.includes(keyword)
    })
    this.setData({ searching: true, filteredItems })
  },

  selectCategory(event) {
    const { id } = event.currentTarget.dataset
    this.setData({ activeCategoryId: id })
    const query = wx.createSelectorQuery()
    query.select(`#cat-${id}`).boundingClientRect()
    query.selectViewport().scrollOffset()
    query.exec((res) => {
      if (!res || !res[0] || !res[1]) {
        return
      }
      const targetTop = res[0].top + res[1].scrollTop
      wx.pageScrollTo({
        scrollTop: Math.max(targetTop - 60, 0),
        duration: 300
      })
    })
  },

  goLogin() {
    wx.navigateTo({
      url: `/pages/customer-login/index?tenantCode=${encodeURIComponent(this.data.tenantCode)}&tenantName=${encodeURIComponent(this.data.tenantName)}`
    })
  },

  // 打开桌号下拉弹层（从门店配置的桌号中选择）
  openDeskPopup() {
    if (this.data.preview) {
      return
    }
    this.setData({ deskPopup: true, deskLoading: true })
    request({
      url: `/desks?tenantCode=${encodeURIComponent(this.data.tenantCode)}`,
      method: 'GET'
    })
      .then((desks) => {
        this.setData({ desks, deskLoading: false })
      })
      .catch((error) => {
        this.setData({ deskLoading: false })
        wx.showToast({ title: error.message || '桌号加载失败，可手动输入', icon: 'none' })
      })
  },

  closeDeskPopup() {
    this.setData({ deskPopup: false })
  },

  selectDesk(event) {
    const { desk } = event.currentTarget.dataset
    if (!desk) {
      return
    }
    this.applyDesk(desk)
  },

  applyDesk(deskCode) {
    const app = getApp()
    app.globalData.deskCode = deskCode
    wx.setStorageSync(`deskCode_${this.data.tenantCode}`, deskCode)
    this.setData({ deskCode, deskPopup: false })
    wx.showToast({ title: `已选择桌号 ${deskCode}`, icon: 'success' })
  },

  // 手动输入兜底（门店未配置桌号或找不到对应桌号时）
  manualDesk() {
    this.setData({ deskPopup: false })
    wx.showModal({
      title: '手动输入桌号',
      editable: true,
      content: this.data.deskCode,
      placeholderText: '请输入餐桌上的桌号，如 A01',
      success: (result) => {
        if (!result.confirm) {
          return
        }
        const deskCode = (result.content || '').trim().toUpperCase()
        if (!deskCode) {
          wx.showToast({ title: '桌号不能为空', icon: 'none' })
          return
        }
        this.applyDesk(deskCode)
      }
    })
  },

  openDishDetail(event) {
    const { id } = event.currentTarget.dataset
    wx.navigateTo({
      url: `/pages/dish-detail/index?tenantCode=${encodeURIComponent(this.data.tenantCode)}&id=${id}`
    })
  },

  logout() {
    wx.removeStorageSync('authToken')
    wx.removeStorageSync('authRole')
    const app = getApp()
    app.globalData.token = ''
    app.globalData.role = ''
    this.refreshCustomer()
    wx.showToast({ title: '已退出登录', icon: 'none' })
  },

  openDishPopup(event) {
    const { id } = event.currentTarget.dataset
    const item = this.data.menuItems.find((menuItem) => menuItem.id === id)
    if (!item) {
      return
    }
    const options = Array.isArray(item.options) ? item.options : []
    this.setData({
      dishPopup: true,
      popupItem: item,
      popupOptions: options,
      popupSelectedFlavor: options.length ? '' : '',
      popupRemark: '',
      popupQuantity: 1
    })
  },

  closeDishPopup() {
    this.setData({ dishPopup: false, popupItem: null })
  },

  stopBubble() {},

  selectFlavor(event) {
    const { flavor } = event.currentTarget.dataset
    this.setData({ popupSelectedFlavor: flavor })
  },

  popupRemarkInput(event) {
    this.setData({ popupRemark: event.detail.value })
  },

  popupQuantityMinus() {
    if (this.data.popupQuantity <= 1) {
      return
    }
    this.setData({ popupQuantity: this.data.popupQuantity - 1 })
  },

  popupQuantityPlus() {
    this.setData({ popupQuantity: this.data.popupQuantity + 1 })
  },

  confirmDishPopup() {
    const { popupItem, popupOptions, popupSelectedFlavor, popupRemark, popupQuantity } = this.data
    if (popupOptions.length && !popupSelectedFlavor) {
      wx.showToast({ title: '请选择口味', icon: 'none' })
      return
    }
    const cart = [...this.data.cart]
    const flavor = popupOptions.length ? popupSelectedFlavor : ''
    const key = `${popupItem.id}|${flavor}|${popupRemark.trim()}`
    const target = cart.find((cartItem) => cartItem.key === key)
    if (target) {
      target.quantity += popupQuantity
    } else {
      cart.push({
        ...popupItem,
        key,
        quantity: popupQuantity,
        flavor,
        remark: popupRemark.trim()
      })
    }
    this.refreshCart(cart)
    this.closeDishPopup()
    wx.showToast({ title: '已加入购物车', icon: 'none' })
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
    if (!this.data.loggedIn) {
      this.goLogin()
      return
    }
    if (!this.data.businessStatus.open) {
      wx.showToast({ title: '门店当前未营业', icon: 'none' })
      return
    }
    if (!this.data.cart.length) {
      wx.showToast({ title: '请先选择菜品', icon: 'none' })
      return
    }
    if (this.data.appendOrderId) {
      try {
        const order = await request({
          url: `/orders/${this.data.appendOrderId}/items`,
          method: 'POST',
          data: {
            items: this.data.cart.map((item) => ({
              menuItemId: item.id,
              quantity: item.quantity,
              flavor: item.flavor || '',
              remark: item.remark || ''
            }))
          }
        })
        this.refreshCart([])
        wx.showToast({ title: '加菜成功', icon: 'success' })
        setTimeout(() => {
          wx.redirectTo({ url: `/pages/order-detail/index?orderId=${order.id}` })
        }, 400)
      } catch (error) {
        wx.showToast({ title: error.message, icon: 'none' })
      }
      return
    }
    if (!this.data.deskCode) {
      wx.showToast({ title: '请先选择桌号', icon: 'none' })
      this.openDeskPopup()
      return
    }
    try {
      const order = await request({
        url: '/orders',
        method: 'POST',
        data: {
          customerName: this.data.customerNickname || this.data.customerPhone,
          customerPhone: this.data.customerPhone,
          deskCode: this.data.deskCode,
          items: this.data.cart.map((item) => ({
            menuItemId: item.id,
            quantity: item.quantity,
            flavor: item.flavor || '',
            remark: item.remark || ''
          }))
        }
      })
      this.refreshCart([])
      wx.showToast({ title: `下单成功，取餐号 ${order.orderNo.slice(-4)}`, icon: 'none' })
      setTimeout(() => {
        wx.navigateTo({ url: `/pages/order-detail/index?orderId=${order.id}` })
      }, 600)
    } catch (error) {
      wx.showToast({ title: error.message, icon: 'none' })
    }
  },

  goHistory() {
    if (!this.data.loggedIn) {
      this.goLogin()
      return
    }
    wx.navigateTo({ url: '/pages/order-history/index' })
  },

  goBackPreview() {
    wx.navigateBack({ delta: 1 })
  }
})
