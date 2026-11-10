const { request } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')

Page({
  data: {
    tenantCode: '',
    status: {
      storeName: '',
      notice: '',
      intro: '',
      open: false
    }
  },

  onShow() {
    const session = requireAdmin()
    if (!session) {
      return
    }
    this.setData({ tenantCode: session.tenantCode })
    this.loadData(session.tenantCode)
  },

  async loadData(tenantCode) {
    try {
      const status = await request({
        url: `/business/status${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
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
      await request({
        url: `/business/status${tenantQuery(this.data.tenantCode)}`,
        method: 'PUT',
        data: this.data.status
      })
      wx.showToast({ title: '保存成功', icon: 'success' })
    } catch (error) {
      wx.showToast({ title: error.message || '保存失败', icon: 'none' })
    }
  },

  backDashboard() {
    wx.navigateTo({ url: '/pages/admin/index' })
  }
})
