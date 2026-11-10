<template>
  <div class="dashboard-page">
    <header class="dashboard-header glass-card">
      <div>
        <p class="welcome">欢迎回来，{{ adminName }}</p>
        <h1>多租户点餐平台 · 超级管理员</h1>
        <p class="sub-copy">管理全部租户、门店二维码、平台经营数据与管理员账号。</p>
      </div>
      <el-button plain @click="logout">退出登录</el-button>
    </header>

    <el-tabs v-model="activeTab" class="console-tabs">
      <el-tab-pane label="全局概览" name="overview">
        <section class="stats-grid">
          <article class="stat-card glass-card">
            <span>租户总数</span>
            <strong>{{ overview.totalTenants || 0 }}</strong>
            <small>启用 {{ overview.enabledTenants || 0 }}</small>
          </article>
          <article class="stat-card glass-card accent">
            <span>累计营收</span>
            <strong>¥ {{ money(overview.totalRevenue) }}</strong>
            <small>今日 ¥ {{ money(overview.todayRevenue) }}</small>
          </article>
          <article class="stat-card glass-card">
            <span>累计订单</span>
            <strong>{{ overview.totalOrders || 0 }}</strong>
            <small>今日 {{ overview.todayOrders || 0 }}</small>
          </article>
          <article class="stat-card glass-card">
            <span>顾客 / 菜品</span>
            <strong>{{ overview.totalCustomers || 0 }} / {{ overview.totalMenuItems || 0 }}</strong>
            <small>全平台统计</small>
          </article>
        </section>

        <section class="glass-card panel">
          <div class="section-title">
            <h3>最近订单</h3>
            <el-button text @click="loadOverview">刷新</el-button>
          </div>
          <el-table :data="overview.recentOrders || []" stripe>
            <el-table-column prop="tenantCode" label="租户" width="100" />
            <el-table-column prop="tenantName" label="门店" min-width="120" />
            <el-table-column label="订单号" min-width="180">
              <template #default="scope">{{ scope.row.order.orderNo }}</template>
            </el-table-column>
            <el-table-column label="顾客" width="110">
              <template #default="scope">{{ scope.row.order.customerName }}</template>
            </el-table-column>
            <el-table-column label="金额" width="100">
              <template #default="scope">¥ {{ money(scope.row.order.totalAmount) }}</template>
            </el-table-column>
            <el-table-column label="状态" width="110">
              <template #default="scope">
                <el-tag>{{ scope.row.order.status }}</el-tag>
              </template>
            </el-table-column>
          </el-table>
        </section>
      </el-tab-pane>

      <el-tab-pane label="租户管理" name="tenants">
        <section class="glass-card panel">
          <div class="section-title">
            <h3>租户列表</h3>
            <div>
              <el-button text @click="loadTenants">刷新</el-button>
              <el-button type="primary" @click="openTenantDialog(null)">新建租户</el-button>
            </div>
          </div>
          <el-table :data="tenants" stripe>
            <el-table-column prop="code" label="编码" width="90" />
            <el-table-column prop="name" label="租户" min-width="130" />
            <el-table-column prop="storeName" label="门店名称" min-width="130" />
            <el-table-column label="营业" width="90">
              <template #default="scope">
                <el-tag :type="scope.row.open ? 'success' : 'info'">{{ scope.row.open ? '营业中' : '打烊' }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="启用" width="90">
              <template #default="scope">
                <el-switch
                  :model-value="scope.row.enabled"
                  @change="(value) => changeTenantEnabled(scope.row, value)"
                />
              </template>
            </el-table-column>
            <el-table-column label="数据" min-width="190">
              <template #default="scope">
                订单 {{ scope.row.orderCount }} · 菜品 {{ scope.row.menuItemCount }} · 管理员 {{ scope.row.adminCount }} · 顾客 {{ scope.row.customerCount }}
              </template>
            </el-table-column>
            <el-table-column label="创建时间" width="170">
              <template #default="scope">{{ formatTime(scope.row.createdTime) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="180" fixed="right">
              <template #default="scope">
                <el-button link type="primary" @click="openTenantDialog(scope.row)">编辑</el-button>
                <el-button link type="primary" @click="showQrcode(scope.row)">二维码</el-button>
              </template>
            </el-table-column>
          </el-table>
        </section>
      </el-tab-pane>

      <el-tab-pane label="账号管理" name="accounts">
        <section class="glass-card panel">
          <div class="section-title">
            <h3>管理员账号</h3>
            <div>
              <el-button text @click="loadAccounts">刷新</el-button>
              <el-button type="primary" @click="openAccountDialog(null)">新增账号</el-button>
            </div>
          </div>
          <el-table :data="accounts" stripe>
            <el-table-column prop="username" label="账号" min-width="130" />
            <el-table-column prop="displayName" label="显示名称" min-width="130" />
            <el-table-column label="角色" width="120">
              <template #default="scope">
                <el-tag :type="scope.row.role === 'SUPER_ADMIN' ? 'danger' : 'success'">
                  {{ scope.row.role === 'SUPER_ADMIN' ? '超级管理员' : '租户管理员' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="所属租户" min-width="150">
              <template #default="scope">
                {{ scope.row.tenantCode ? `${scope.row.tenantCode} ${scope.row.tenantName}` : '平台级' }}
              </template>
            </el-table-column>
            <el-table-column label="启用" width="90">
              <template #default="scope">
                <el-switch
                  :model-value="scope.row.enabled"
                  @change="(value) => changeAccountEnabled(scope.row, value)"
                />
              </template>
            </el-table-column>
            <el-table-column label="创建时间" width="170">
              <template #default="scope">{{ formatTime(scope.row.createdTime) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="scope">
                <el-button link type="primary" @click="openAccountDialog(scope.row)">编辑</el-button>
                <el-button link type="danger" @click="removeAccount(scope.row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </section>
      </el-tab-pane>
    </el-tabs>

    <el-dialog v-model="tenantDialogVisible" :title="tenantForm.id ? '编辑租户' : '新建租户'" width="560px">
      <el-form :model="tenantForm" label-position="top">
        <el-form-item v-if="!tenantForm.id" label="租户编码">
          <el-input v-model="tenantForm.code" placeholder="例如 T005，系统内唯一" />
        </el-form-item>
        <el-form-item label="租户名称"><el-input v-model="tenantForm.name" /></el-form-item>
        <el-form-item label="门店名称"><el-input v-model="tenantForm.storeName" /></el-form-item>
        <el-form-item label="店铺介绍"><el-input v-model="tenantForm.intro" type="textarea" :rows="2" /></el-form-item>
        <el-form-item label="营业公告"><el-input v-model="tenantForm.notice" type="textarea" :rows="2" /></el-form-item>
        <el-form-item v-if="!tenantForm.id" label="初始租户管理员（可选）">
          <el-input v-model="tenantForm.adminUsername" placeholder="管理员账号" />
        </el-form-item>
        <el-form-item v-if="!tenantForm.id && tenantForm.adminUsername" label="管理员密码">
          <el-input v-model="tenantForm.adminPassword" placeholder="至少 6 位" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tenantDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitTenant">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="qrcodeVisible" title="门店二维码" width="420px">
      <div class="qrcode-box">
        <img v-if="qrcode" :src="qrcode.url" alt="门店二维码" class="qrcode-image" />
        <p>{{ qrcodeTenantCode }}</p>
        <p class="qrcode-hint">扫码后进入该租户的点餐页面</p>
      </div>
    </el-dialog>

    <el-dialog v-model="accountDialogVisible" :title="accountForm.id ? '编辑账号' : '新增账号'" width="520px">
      <el-form :model="accountForm" label-position="top">
        <el-form-item v-if="!accountForm.id" label="账号"><el-input v-model="accountForm.username" /></el-form-item>
        <el-form-item label="显示名称"><el-input v-model="accountForm.displayName" /></el-form-item>
        <el-form-item v-if="!accountForm.id" label="角色">
          <el-select v-model="accountForm.role" style="width: 100%">
            <el-option label="超级管理员" value="SUPER_ADMIN" />
            <el-option label="租户管理员" value="TENANT_ADMIN" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="!accountForm.id && accountForm.role === 'TENANT_ADMIN'" label="所属租户">
          <el-select v-model="accountForm.tenantId" style="width: 100%" placeholder="请选择租户">
            <el-option
              v-for="tenant in tenants"
              :key="tenant.id"
              :label="`${tenant.code} ${tenant.name}`"
              :value="tenant.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item :label="accountForm.id ? '重置密码（不修改请留空）' : '登录密码'">
          <el-input v-model="accountForm.password" show-password placeholder="至少 6 位" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="accountDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitAccount">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  adminLogout,
  createAccount,
  createTenant,
  deleteAccount,
  getOverview,
  getTenantQrcode,
  listAccounts,
  listTenants,
  updateAccount,
  updateTenant
} from '../api'

const router = useRouter()
const adminName = localStorage.getItem('admin_name') || '超级管理员'
const activeTab = ref('overview')
const overview = ref({})
const tenants = ref([])
const accounts = ref([])

const tenantDialogVisible = ref(false)
const qrcodeVisible = ref(false)
const qrcode = ref(null)
const qrcodeTenantCode = ref('')
const accountDialogVisible = ref(false)

const tenantForm = reactive(resetTenantForm())
const accountForm = reactive(resetAccountForm())

function resetTenantForm() {
  return {
    id: null,
    code: '',
    name: '',
    storeName: '',
    intro: '',
    notice: '',
    adminUsername: '',
    adminPassword: ''
  }
}

function resetAccountForm() {
  return {
    id: null,
    username: '',
    displayName: '',
    role: 'TENANT_ADMIN',
    tenantId: null,
    password: ''
  }
}

const money = (value) => Number(value || 0).toFixed(2)
const formatTime = (value) => (value ? String(value).replace('T', ' ').slice(0, 19) : '')

const loadOverview = async () => {
  overview.value = await getOverview()
}

const loadTenants = async () => {
  tenants.value = await listTenants()
}

const loadAccounts = async () => {
  accounts.value = await listAccounts()
}

const loadAll = async () => {
  try {
    await Promise.all([loadOverview(), loadTenants(), loadAccounts()])
  } catch (error) {
    ElMessage.error(error.message || '数据加载失败')
  }
}

const openTenantDialog = (row) => {
  Object.assign(tenantForm, resetTenantForm(), row ? {
    id: row.id,
    code: row.code,
    name: row.name,
    storeName: row.storeName,
    intro: row.intro,
    notice: row.notice
  } : {})
  tenantDialogVisible.value = true
}

const submitTenant = async () => {
  if (!tenantForm.name) {
    ElMessage.warning('请填写租户名称')
    return
  }
  try {
    if (tenantForm.id) {
      await updateTenant(tenantForm.id, {
        name: tenantForm.name,
        storeName: tenantForm.storeName,
        intro: tenantForm.intro,
        notice: tenantForm.notice
      })
      ElMessage.success('租户已更新')
    } else {
      await createTenant({
        code: tenantForm.code,
        name: tenantForm.name,
        storeName: tenantForm.storeName,
        intro: tenantForm.intro,
        notice: tenantForm.notice,
        adminUsername: tenantForm.adminUsername,
        adminPassword: tenantForm.adminPassword
      })
      ElMessage.success('租户创建成功')
    }
    tenantDialogVisible.value = false
    await Promise.all([loadTenants(), loadAccounts(), loadOverview()])
  } catch (error) {
    ElMessage.error(error.message || '保存失败')
  }
}

const changeTenantEnabled = async (row, enabled) => {
  try {
    await updateTenant(row.id, { enabled })
    ElMessage.success(enabled ? '租户已启用' : '租户已停用')
  } catch (error) {
    ElMessage.error(error.message || '操作失败')
  }
  await Promise.all([loadTenants(), loadOverview()])
}

const showQrcode = async (row) => {
  try {
    qrcode.value = await getTenantQrcode(row.id)
    qrcodeTenantCode.value = row.code
    qrcodeVisible.value = true
  } catch (error) {
    ElMessage.error(error.message || '二维码生成失败')
  }
}

const openAccountDialog = (row) => {
  Object.assign(accountForm, resetAccountForm(), row ? {
    id: row.id,
    username: row.username,
    displayName: row.displayName,
    role: row.role,
    tenantId: row.tenantId,
    password: ''
  } : {})
  accountDialogVisible.value = true
}

const submitAccount = async () => {
  try {
    if (accountForm.id) {
      const payload = { displayName: accountForm.displayName }
      if (accountForm.password) {
        payload.password = accountForm.password
      }
      await updateAccount(accountForm.id, payload)
      ElMessage.success('账号已更新')
    } else {
      await createAccount({
        username: accountForm.username,
        displayName: accountForm.displayName,
        role: accountForm.role,
        tenantId: accountForm.tenantId,
        password: accountForm.password
      })
      ElMessage.success('账号创建成功')
    }
    accountDialogVisible.value = false
    await Promise.all([loadAccounts(), loadTenants()])
  } catch (error) {
    ElMessage.error(error.message || '保存失败')
  }
}

const changeAccountEnabled = async (row, enabled) => {
  try {
    await updateAccount(row.id, { enabled })
    ElMessage.success(enabled ? '账号已启用' : '账号已停用')
  } catch (error) {
    ElMessage.error(error.message || '操作失败')
  }
  await loadAccounts()
}

const removeAccount = async (row) => {
  await ElMessageBox.confirm(`确定删除账号 ${row.username} 吗？`, '删除确认', { type: 'warning' })
  try {
    await deleteAccount(row.id)
    ElMessage.success('账号已删除')
    await Promise.all([loadAccounts(), loadTenants()])
  } catch (error) {
    ElMessage.error(error.message || '删除失败')
  }
}

const logout = async () => {
  try {
    await adminLogout()
  } catch (error) {
    // 令牌可能已过期，继续清理本地登录态
  }
  localStorage.removeItem('admin_token')
  localStorage.removeItem('admin_name')
  localStorage.removeItem('admin_role')
  router.push('/login')
}

onMounted(loadAll)
</script>

<style scoped>
.dashboard-page {
  padding: 24px;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 28px;
  margin-bottom: 24px;
}

.welcome {
  color: #8d684f;
  margin: 0 0 10px;
}

.dashboard-header h1 {
  margin: 0;
  font-size: 36px;
}

.sub-copy {
  color: #755948;
  max-width: 640px;
  line-height: 1.8;
}

.console-tabs {
  margin-bottom: 24px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 18px;
  margin-bottom: 24px;
}

.stat-card {
  padding: 24px;
}

.stat-card span {
  display: block;
  color: #816552;
  margin-bottom: 12px;
}

.stat-card strong {
  font-size: 32px;
}

.stat-card small {
  display: block;
  margin-top: 8px;
  color: #816552;
}

.accent {
  background: linear-gradient(135deg, rgba(182, 84, 43, 0.95), rgba(103, 40, 16, 0.92));
  color: #fff6ee;
}

.accent span,
.accent small {
  color: rgba(255, 246, 238, 0.8);
}

.panel {
  padding: 24px;
  margin-bottom: 24px;
}

.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
}

.section-title h3 {
  margin: 0;
}

.qrcode-box {
  text-align: center;
}

.qrcode-image {
  width: 220px;
  height: 220px;
}

.qrcode-hint {
  color: #816552;
  font-size: 13px;
}

@media (max-width: 1100px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
