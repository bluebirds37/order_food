<template>
  <div class="login-page">
    <div class="login-panel glass-card">
      <div class="hero-copy">
        <p class="eyebrow">Order Food · Platform</p>
        <h1>多租户点餐平台</h1>
        <p>超级管理员可管理全部租户、生成门店二维码、查看全局经营数据与账号权限。</p>
      </div>
      <el-form :model="form" class="login-form" @submit.prevent="handleLogin">
        <el-form-item>
          <el-input v-model="form.username" size="large" placeholder="超级管理员账号" />
        </el-form-item>
        <el-form-item>
          <el-input v-model="form.password" size="large" type="password" show-password placeholder="请输入密码" />
        </el-form-item>
        <el-button type="primary" size="large" class="submit-button" @click="handleLogin">登录平台后台</el-button>
        <p class="hint">仅超级管理员可登录；租户管理员请使用小程序管理端。</p>
      </el-form>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { adminLogin } from '../api'

const router = useRouter()
const form = reactive({ username: '', password: '' })

const handleLogin = async () => {
  try {
    const data = await adminLogin(form)
    if (data.role !== 'SUPER_ADMIN') {
      ElMessage.error('该账号不是超级管理员，无法登录 PC 端')
      return
    }
    localStorage.setItem('admin_token', data.token)
    localStorage.setItem('admin_name', data.displayName)
    localStorage.setItem('admin_role', data.role)
    ElMessage.success('登录成功')
    router.push('/dashboard')
  } catch (error) {
    ElMessage.error(error.message || '登录失败')
  }
}
</script>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.login-panel {
  width: min(1100px, 100%);
  padding: 36px;
  display: grid;
  grid-template-columns: 1.2fr 0.8fr;
  gap: 28px;
}

.hero-copy {
  min-height: 420px;
  padding: 30px;
  border-radius: 22px;
  color: #fff7f0;
  background:
    linear-gradient(135deg, rgba(66, 27, 9, 0.82), rgba(182, 84, 43, 0.75)),
    url('https://images.unsplash.com/photo-1552566626-52f8b828add9?auto=format&fit=crop&w=1200&q=80') center/cover;
}

.eyebrow {
  margin: 0 0 16px;
  letter-spacing: 4px;
  text-transform: uppercase;
  opacity: 0.85;
}

.hero-copy h1 {
  margin: 0 0 18px;
  font-size: 48px;
  line-height: 1.15;
}

.hero-copy p:last-child {
  max-width: 420px;
  font-size: 16px;
  line-height: 1.8;
}

.login-form {
  padding: 32px;
  align-self: center;
}

.submit-button {
  width: 100%;
  height: 48px;
  border: none;
  background: linear-gradient(135deg, #c56a3d, #8d3818);
}

.hint {
  margin-top: 12px;
  color: #82614f;
}

@media (max-width: 900px) {
  .login-panel {
    grid-template-columns: 1fr;
  }

  .hero-copy h1 {
    font-size: 36px;
  }
}
</style>
