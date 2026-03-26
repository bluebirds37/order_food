import axios from 'axios'

const request = axios.create({
  baseURL: 'http://localhost:8080/api',
  timeout: 10000
})

request.interceptors.response.use(
  (response) => {
    const payload = response.data
    if (!payload.success) {
      return Promise.reject(new Error(payload.message))
    }
    return payload.data
  },
  (error) => Promise.reject(error)
)

export const adminLogin = (data) => request.post('/admin/login', data)
export const getDashboard = () => request.get('/admin/dashboard')
export const getBusinessStatus = () => request.get('/business/status')
export const updateBusinessStatus = (data) => request.put('/business/status', data)
export const getMenuItems = () => request.get('/menu/items')
export const getMenuOverview = () => request.get('/menu/overview')
export const createMenuItem = (data) => request.post('/menu/items', data)
export const updateMenuItem = (id, data) => request.put(`/menu/items/${id}`, data)
export const deleteMenuItem = (id) => request.delete(`/menu/items/${id}`)
export const getOrders = () => request.get('/orders')
export const updateOrderStatus = (id, data) => request.put(`/orders/${id}/status`, data)
