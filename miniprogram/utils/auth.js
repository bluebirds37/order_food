const { request } = require('./request')

// 微信无感登录：wx.login 换 code，后端换 openid 并自动注册/登录
// 加超时兜底，避免 wx.login 无响应时页面一直停留在"正在登录"
function silentWxLogin(tenantCode) {
  return new Promise((resolve, reject) => {
    let settled = false
    const finish = (fn, arg) => {
      if (settled) {
        return
      }
      settled = true
      clearTimeout(timer)
      fn(arg)
    }
    const timer = setTimeout(() => {
      finish(reject, new Error('登录超时，请点击按钮重试'))
    }, 10000)

    wx.login({
      success: (res) => {
        if (settled) {
          return
        }
        if (!res.code) {
          finish(reject, new Error('微信登录失败，请重试'))
          return
        }
        request({
          url: '/auth/customer/wx-login',
          method: 'POST',
          data: { tenantCode, code: res.code }
        })
          .then((data) => {
            wx.setStorageSync('authToken', data.token)
            wx.setStorageSync('authRole', 'CUSTOMER')
            wx.setStorageSync('customerNickname', data.nickname)
            wx.setStorageSync('customerPhone', data.phone)
            wx.setStorageSync('tenantCode', data.tenantCode)
            const app = getApp()
            app.globalData.token = data.token
            app.globalData.role = 'CUSTOMER'
            finish(resolve, data)
          })
          .catch((error) => finish(reject, error))
      },
      fail: () => finish(reject, new Error('微信登录调用失败'))
    })
  })
}

// wx 无感登录后手机号是占位符，需要引导一键绑定
function isPlaceholderPhone(phone) {
  return !phone || phone.indexOf('wx_') === 0
}

module.exports = {
  silentWxLogin,
  isPlaceholderPhone
}
