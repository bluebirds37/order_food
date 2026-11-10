const { request } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')
const { orderStatusText } = require('../../utils/status')

const NEXT_ACTION = {
  PAID: { next: 'PREPARING', label: '开始制作' },
  PREPARING: { next: 'COMPLETED', label: '完成出餐' }
}

Page({
  data: {
    tenantCode: '',
    loading: false,
    desks: [],
    deskCount: 0,
    orderCount: 0,
    dishCount: 0
  },

  onShow() {
    const session = requireAdmin()
    if (!session) {
      return
    }
    this.setData({ tenantCode: session.tenantCode })
    this.loadBoard(session.tenantCode)
  },

  onPullDownRefresh() {
    this.loadBoard(this.data.tenantCode).finally(() => wx.stopPullDownRefresh())
  },

  async loadBoard(tenantCode) {
    this.setData({ loading: true })
    try {
      const desks = await request({
        url: `/kitchen/pending${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
      const view = desks.map((desk) => ({
        ...desk,
        orders: desk.orders.map((order) => ({
          ...order,
          statusText: orderStatusText(order.status),
          pickupNo: (order.orderNo || '').slice(-4),
          action: NEXT_ACTION[order.status] || null
        }))
      }))
      const orderCount = view.reduce((sum, desk) => sum + desk.orderCount, 0)
      const dishCount = view.reduce(
        (sum, desk) => sum + desk.mergedItems.reduce((s, item) => s + item.quantity, 0),
        0
      )
      this.setData({ desks: view, deskCount: view.length, orderCount, dishCount })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  advanceOrder(event) {
    const { id, next } = event.currentTarget.dataset
    const nextStatus = String(next)
    wx.showLoading({ title: '处理中' })
    request({
      url: `/orders/${id}/status${tenantQuery(this.data.tenantCode)}`,
      method: 'PUT',
      data: { status: nextStatus }
    })
      .then(() => {
        wx.hideLoading()
        wx.showToast({
          title: nextStatus === 'PREPARING' ? '已开始制作' : '已出餐完成',
          icon: 'success'
        })
        this.loadBoard(this.data.tenantCode)
      })
      .catch((error) => {
        wx.hideLoading()
        wx.showToast({ title: error.message || '操作失败', icon: 'none' })
      })
  }
})
