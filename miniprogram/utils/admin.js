const ADMIN_ROLES = ['TENANT_ADMIN', 'SUPER_ADMIN']

function requireAdmin() {
  const token = wx.getStorageSync('authToken')
  const role = wx.getStorageSync('authRole')
  if (!token || ADMIN_ROLES.indexOf(role) === -1) {
    wx.redirectTo({ url: '/pages/admin-login/index' })
    return null
  }
  const app = getApp()
  const tenantCode = app.globalData.tenantCode || wx.getStorageSync('tenantCode')
  if (!tenantCode) {
    wx.showToast({ title: '请先选择门店', icon: 'none' })
    setTimeout(() => {
      wx.redirectTo({ url: '/pages/entry/index' })
    }, 300)
    return null
  }
  return { token, role, tenantCode }
}

function tenantQuery(tenantCode) {
  return `?tenantCode=${encodeURIComponent(tenantCode)}`
}

module.exports = {
  requireAdmin,
  tenantQuery
}
