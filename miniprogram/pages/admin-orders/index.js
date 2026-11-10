const { request } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')
const { withStatusText } = require('../../utils/status')

const STATUS_OPTIONS = ['PAID', 'PREPARING', 'COMPLETED', 'CANCELLED']

Page({
  data: {
    tenantCode: '',
    loading: false,
    orders: [],
    statusOptions: STATUS_OPTIONS
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
    this.setData({ loading: true })
    try {
      const orders = await request({
        url: `/orders${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
      this.setData({ orders: withStatusText(orders) })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  confirmTurnover(event) {
    const { id } = event.currentTarget.dataset
    wx.showModal({
      title: '确认翻台',
      content: '翻台后本轮消费结束，顾客将无法继续追加菜品，是否继续？',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        try {
          await request({
            url: `/orders/${id}/status${tenantQuery(this.data.tenantCode)}`,
            method: 'PUT',
            data: { status: 'COMPLETED' }
          })
          wx.showToast({ title: '已翻台', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '翻台失败', icon: 'none' })
        }
      }
    })
  },

  async updateStatus(event) {
    const { id, status } = event.currentTarget.dataset
    try {
      await request({
        url: `/orders/${id}/status${tenantQuery(this.data.tenantCode)}`,
        method: 'PUT',
        data: { status }
      })
      wx.showToast({ title: '状态已更新', icon: 'success' })
      this.loadData(this.data.tenantCode)
    } catch (error) {
      wx.showToast({ title: error.message || '更新失败', icon: 'none' })
    }
  }
})
