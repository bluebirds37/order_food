const request = (options) => {
  const app = getApp()
  return new Promise((resolve, reject) => {
    wx.request({
      ...options,
      url: `${app.globalData.baseUrl}${options.url}`,
      success: (res) => {
        const data = res.data
        if (!data.success) {
          reject(new Error(data.message || '请求失败'))
          return
        }
        resolve(data.data)
      },
      fail: reject
    })
  })
}

module.exports = {
  request
}
