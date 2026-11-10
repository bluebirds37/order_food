import axios from 'axios'

const request = axios.create({
  baseURL: 'http://localhost:8080/api',
  timeout: 10000
})

request.interceptors.request.use((config) => {
  const token = localStorage.getItem('admin_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

request.interceptors.response.use(
  (response) => {
    const payload = response.data
    if (!payload.success) {
      return Promise.reject(new Error(payload.message))
    }
    return payload.data
  },
  (error) => {
    const message = error.response && error.response.data && error.response.data.message
    if (error.response && error.response.status === 401) {
      localStorage.removeItem('admin_token')
    }
    return Promise.reject(new Error(message || error.message || '请求失败'))
  }
)

export const adminLogin = (data) => request.post('/admin/login', data)
export const adminLogout = () => request.post('/admin/logout')

export const getOverview = () => request.get('/super/overview')
export const listTenants = () => request.get('/super/tenants')
export const createTenant = (data) => request.post('/super/tenants', data)
export const updateTenant = (id, data) => request.put(`/super/tenants/${id}`, data)
export const getTenantQrcode = (id) => request.get(`/super/tenants/${id}/qrcode`)

export const listAccounts = (params) => request.get('/super/accounts', { params })
export const createAccount = (data) => request.post('/super/accounts', data)
export const updateAccount = (id, data) => request.put(`/super/accounts/${id}`, data)
export const deleteAccount = (id) => request.delete(`/super/accounts/${id}`)
