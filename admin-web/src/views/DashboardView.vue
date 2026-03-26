<template>
  <div class="dashboard-page">
    <header class="dashboard-header glass-card">
      <div>
        <p class="welcome">欢迎回来，{{ adminName }}</p>
        <h1>{{ dashboard.storeName || '食光小馆' }}经营中心</h1>
        <p class="sub-copy">聚焦营业状态、菜单供给与订单履约，适合餐饮门店的轻量化经营后台。</p>
      </div>
      <el-button plain @click="logout">退出登录</el-button>
    </header>

    <section class="stats-grid">
      <article class="stat-card glass-card">
        <span>今日订单</span>
        <strong>{{ dashboard.todayOrderCount || 0 }}</strong>
      </article>
      <article class="stat-card glass-card accent">
        <span>今日营收</span>
        <strong>¥ {{ Number(dashboard.todayRevenue || 0).toFixed(2) }}</strong>
      </article>
      <article class="stat-card glass-card">
        <span>在售菜品</span>
        <strong>{{ dashboard.availableMenuCount || 0 }}</strong>
      </article>
      <article class="stat-card glass-card">
        <span>当前状态</span>
        <strong>{{ businessForm.open ? '营业中' : '已打烊' }}</strong>
      </article>
    </section>

    <section class="content-grid">
      <div class="glass-card status-panel">
        <div class="section-title">
          <h3>营业设置</h3>
          <el-tag :type="businessForm.open ? 'success' : 'danger'">{{ businessForm.open ? '营业中' : '休息中' }}</el-tag>
        </div>
        <el-form :model="businessForm" label-position="top">
          <el-form-item label="门店名称">
            <el-input v-model="businessForm.storeName" />
          </el-form-item>
          <el-form-item label="营业公告">
            <el-input v-model="businessForm.notice" type="textarea" :rows="3" />
          </el-form-item>
          <el-form-item label="营业开关">
            <el-switch v-model="businessForm.open" size="large" inline-prompt active-text="营业" inactive-text="打烊" />
          </el-form-item>
          <el-button type="primary" @click="saveBusiness">保存营业设置</el-button>
        </el-form>
      </div>

      <div class="glass-card menu-panel">
        <div class="section-title">
          <h3>菜单管理</h3>
          <el-button type="primary" @click="openDialog(null)">新增菜品</el-button>
        </div>
        <el-table :data="menuItems" stripe>
          <el-table-column prop="name" label="菜品" min-width="140" />
          <el-table-column prop="categoryName" label="分类" width="110" />
          <el-table-column prop="price" label="价格" width="90">
            <template #default="scope">¥ {{ scope.row.price }}</template>
          </el-table-column>
          <el-table-column label="状态" width="90">
            <template #default="scope">
              <el-tag :type="scope.row.available ? 'success' : 'info'">{{ scope.row.available ? '在售' : '停售' }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="180">
            <template #default="scope">
              <el-button link type="primary" @click="openDialog(scope.row)">编辑</el-button>
              <el-button link type="danger" @click="removeMenuItem(scope.row.id)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </section>

    <section class="glass-card order-panel">
      <div class="section-title">
        <h3>最新订单</h3>
      </div>
      <el-table :data="orders" stripe>
        <el-table-column prop="orderNo" label="订单号" min-width="180" />
        <el-table-column prop="customerName" label="顾客" width="100" />
        <el-table-column prop="deskCode" label="桌号" width="90" />
        <el-table-column prop="totalAmount" label="金额" width="90">
          <template #default="scope">¥ {{ scope.row.totalAmount }}</template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="130" />
        <el-table-column label="操作" width="180">
          <template #default="scope">
            <el-select :model-value="scope.row.status" @change="(value) => changeOrderStatus(scope.row.id, value)">
              <el-option label="已支付" value="PAID" />
              <el-option label="制作中" value="PREPARING" />
              <el-option label="已完成" value="COMPLETED" />
              <el-option label="已取消" value="CANCELLED" />
            </el-select>
          </template>
        </el-table-column>
      </el-table>
    </section>

    <el-dialog v-model="dialogVisible" title="菜品信息" width="560px">
      <el-form :model="menuForm" label-position="top">
        <el-form-item label="菜品名称"><el-input v-model="menuForm.name" /></el-form-item>
        <el-form-item label="菜品分类">
          <el-select v-model="menuForm.categoryId" style="width: 100%">
            <el-option v-for="item in categoryOptions" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="价格"><el-input v-model="menuForm.price" /></el-form-item>
        <el-form-item label="图片地址"><el-input v-model="menuForm.imageUrl" /></el-form-item>
        <el-form-item label="菜品描述"><el-input v-model="menuForm.description" type="textarea" :rows="3" /></el-form-item>
        <el-form-item label="排序"><el-input-number v-model="menuForm.sortOrder" :min="1" /></el-form-item>
        <el-form-item>
          <el-switch v-model="menuForm.recommended" inline-prompt active-text="推荐" inactive-text="普通" />
          <el-switch v-model="menuForm.available" inline-prompt active-text="在售" inactive-text="停售" style="margin-left: 12px" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitMenuForm">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  createMenuItem,
  deleteMenuItem,
  getBusinessStatus,
  getDashboard,
  getMenuItems,
  getMenuOverview,
  getOrders,
  updateBusinessStatus,
  updateMenuItem,
  updateOrderStatus
} from '../api'

const router = useRouter()
const adminName = localStorage.getItem('admin_name') || '管理员'
const dashboard = ref({})
const orders = ref([])
const menuItems = ref([])
const categoryOptions = ref([])
const dialogVisible = ref(false)
const menuForm = reactive(resetMenuForm())
const businessForm = reactive({ storeName: '', notice: '', open: true })

function resetMenuForm() {
  return {
    id: null,
    categoryId: null,
    name: '',
    price: '0.00',
    description: '',
    imageUrl: '',
    recommended: false,
    available: true,
    sortOrder: 1
  }
}

const fillMenuForm = (row) => {
  Object.assign(menuForm, row ? { ...row } : resetMenuForm())
}

const loadData = async () => {
  const [dashboardData, statusData, menuData, menuOverview, orderData] = await Promise.all([
    getDashboard(),
    getBusinessStatus(),
    getMenuItems(),
    getMenuOverview(),
    getOrders()
  ])
  dashboard.value = dashboardData
  orders.value = orderData
  menuItems.value = menuData
  categoryOptions.value = menuOverview.categories
  Object.assign(businessForm, statusData)
}

const saveBusiness = async () => {
  await updateBusinessStatus(businessForm)
  ElMessage.success('营业设置已保存')
  await loadData()
}

const openDialog = (row) => {
  fillMenuForm(row)
  dialogVisible.value = true
}

const submitMenuForm = async () => {
  const payload = {
    ...menuForm,
    price: Number(menuForm.price)
  }
  if (menuForm.id) {
    await updateMenuItem(menuForm.id, payload)
  } else {
    await createMenuItem(payload)
  }
  dialogVisible.value = false
  ElMessage.success('菜品已保存')
  await loadData()
}

const removeMenuItem = async (id) => {
  await ElMessageBox.confirm('删除后将不再展示该菜品，是否继续？', '删除确认', { type: 'warning' })
  await deleteMenuItem(id)
  ElMessage.success('删除成功')
  await loadData()
}

const changeOrderStatus = async (id, status) => {
  await updateOrderStatus(id, { status })
  ElMessage.success('订单状态已更新')
  await loadData()
}

const logout = () => {
  localStorage.removeItem('admin_token')
  localStorage.removeItem('admin_name')
  router.push('/login')
}

onMounted(async () => {
  try {
    await loadData()
  } catch (error) {
    ElMessage.error(error.message || '数据加载失败')
  }
})
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
  font-size: 38px;
}

.sub-copy {
  color: #755948;
  max-width: 640px;
  line-height: 1.8;
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

.accent {
  background: linear-gradient(135deg, rgba(182, 84, 43, 0.95), rgba(103, 40, 16, 0.92));
  color: #fff6ee;
}

.accent span {
  color: rgba(255, 246, 238, 0.8);
}

.content-grid {
  display: grid;
  grid-template-columns: 360px 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

.status-panel,
.menu-panel,
.order-panel {
  padding: 24px;
}

@media (max-width: 1100px) {
  .stats-grid,
  .content-grid {
    grid-template-columns: 1fr;
  }
}
</style>
