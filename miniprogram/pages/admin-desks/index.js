const { request } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')

Page({
  data: {
    tenantCode: '',
    loading: false,
    desks: []
  },

  onShow() {
    const session = requireAdmin()
    if (!session) {
      return
    }
    this.setData({ tenantCode: session.tenantCode })
    this.loadDesks(session.tenantCode)
  },

  async loadDesks(tenantCode) {
    this.setData({ loading: true })
    try {
      const desks = await request({
        url: `/desks${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
      this.setData({ desks })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  addDesk() {
    wx.showModal({
      title: '新增桌号',
      editable: true,
      placeholderText: '请输入桌号，如 A07',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        const deskCode = (result.content || '').trim().toUpperCase()
        if (!deskCode) {
          wx.showToast({ title: '桌号不能为空', icon: 'none' })
          return
        }
        try {
          await request({
            url: `/desks${tenantQuery(this.data.tenantCode)}`,
            method: 'POST',
            data: { deskCode }
          })
          wx.showToast({ title: '桌号已新增', icon: 'success' })
          this.loadDesks(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '新增失败', icon: 'none' })
        }
      }
    })
  },

  editDesk(event) {
    const { id, code } = event.currentTarget.dataset
    wx.showModal({
      title: '修改桌号',
      editable: true,
      content: code,
      placeholderText: '请输入新桌号',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        const deskCode = (result.content || '').trim().toUpperCase()
        if (!deskCode || deskCode === code) {
          return
        }
        try {
          await request({
            url: `/desks/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'PUT',
            data: { deskCode }
          })
          wx.showToast({ title: '桌号已更新', icon: 'success' })
          this.loadDesks(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '更新失败', icon: 'none' })
        }
      }
    })
  },

  deleteDesk(event) {
    const { id, code } = event.currentTarget.dataset
    wx.showModal({
      title: '删除桌号',
      content: `删除后顾客将无法再选择桌号 ${code}，是否继续？`,
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        try {
          await request({
            url: `/desks/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'DELETE'
          })
          wx.showToast({ title: '删除成功', icon: 'success' })
          this.loadDesks(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '删除失败', icon: 'none' })
        }
      }
    })
  }
})
