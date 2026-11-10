import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import DashboardView from '../views/DashboardView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/dashboard' },
    { path: '/login', component: LoginView },
    { path: '/dashboard', component: DashboardView }
  ]
})

router.beforeEach((to) => {
  const token = localStorage.getItem('admin_token')
  const role = localStorage.getItem('admin_role')
  const isSuperAdmin = token && role === 'SUPER_ADMIN'
  if (to.path !== '/login' && !isSuperAdmin) {
    return '/login'
  }
  if (to.path === '/login' && isSuperAdmin) {
    return '/dashboard'
  }
})

export default router
