const request = (options) => {
  const app = getApp()
  const token = wx.getStorageSync('authToken') || ''
  const header = Object.assign({}, options.header || {})
  if (token) {
    header.Authorization = `Bearer ${token}`
  }
  return new Promise((resolve, reject) => {
    wx.request({
      ...options,
      timeout: options.timeout || 10000,
      header,
      url: `${app.globalData.baseUrl}${options.url}`,
      success: (res) => {
        const data = res.data
        if (!data || !data.success) {
          reject(new Error((data && data.message) || '请求失败'))
          return
        }
        resolve(data.data)
      },
      fail: (error) => {
        reject(new Error(error && error.errMsg && error.errMsg.indexOf('timeout') >= 0 ? '请求超时，请检查后端服务后重试' : '网络请求失败，请检查后端服务'))
      }
    })
  })
}

const uploadImage = (filePath) => {
  const app = getApp()
  const token = wx.getStorageSync('authToken') || ''
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${app.globalData.baseUrl}/images`,
      filePath,
      name: 'file',
      header: token ? { Authorization: `Bearer ${token}` } : {},
      success: (res) => {
        let payload
        try {
          payload = JSON.parse(res.data)
        } catch (error) {
          reject(new Error('图片上传响应解析失败'))
          return
        }
        if (!payload || !payload.success) {
          reject(new Error((payload && payload.message) || '图片上传失败'))
          return
        }
        resolve(payload.data)
      },
      fail: reject
    })
  })
}

module.exports = {
  request,
  uploadImage
}
