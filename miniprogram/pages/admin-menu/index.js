const { request } = require('../../utils/request')

function createDefaultForm() {
  return {
    id: null,
    categoryId: 1,
    name: '',
    price: '0.00',
    imageUrl: '',
    description: '',
    sortOrder: 1,
    recommended: false,
    available: true
  }
}

Page({
  data: {
    loading: false,
    menuItems: [],
    categories: [],
    showEditor: false,
    currentCategoryName: '',
    form: createDefaultForm()
  },

  onShow() {
    if (!wx.getStorageSync('adminToken')) {
      wx.redirectTo({ url: '/pages/admin-login/index' })
      return
    }
    this.loadData()
  },

  async loadData() {
    this.setData({ loading: true })
    try {
      const overview = await request({ url: '/menu/overview', method: 'GET' })
      const currentCategoryName = overview.categories.length ? overview.categories[0].name : ''
      this.setData({
        menuItems: overview.menuItems,
        categories: overview.categories,
        currentCategoryName
      })
    } catch (error) {
      wx.showToast({ title: error.message || '加载失败', icon: 'none' })
    } finally {
      this.setData({ loading: false })
    }
  },

  openCreate() {
    const defaultCategoryId = this.data.categories.length ? this.data.categories[0].id : 1
    this.setData({
      showEditor: true,
      currentCategoryName: this.data.categories.length ? this.data.categories[0].name : '',
      form: {
        ...createDefaultForm(),
        categoryId: defaultCategoryId
      }
    })
  },

  openEdit(event) {
    const { id } = event.currentTarget.dataset
    const target = this.data.menuItems.find((item) => item.id === id)
    if (!target) {
      return
    }
    this.setData({
      showEditor: true,
      currentCategoryName: target.categoryName,
      form: {
        id: target.id,
        categoryId: target.categoryId,
        name: target.name,
        price: String(target.price),
        imageUrl: target.imageUrl,
        description: target.description,
        sortOrder: target.sortOrder,
        recommended: target.recommended,
        available: target.available
      }
    })
  },

  closeEditor() {
    this.setData({
      showEditor: false,
      currentCategoryName: this.data.categories.length ? this.data.categories[0].name : '',
      form: createDefaultForm()
    })
  },

  handleInput(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [`form.${field}`]: event.detail.value })
  },

  handleSwitch(event) {
    const { field } = event.currentTarget.dataset
    this.setData({ [`form.${field}`]: event.detail.value })
  },

  handleCategoryChange(event) {
    const index = Number(event.detail.value)
    const category = this.data.categories[index]
    if (!category) {
      return
    }
    this.setData({
      currentCategoryName: category.name,
      'form.categoryId': category.id
    })
  },

  async submitForm() {
    const { form } = this.data
    if (!form.name || !form.description || !form.imageUrl) {
      wx.showToast({ title: '请补全菜品信息', icon: 'none' })
      return
    }
    try {
      const payload = {
        ...form,
        price: Number(form.price),
        sortOrder: Number(form.sortOrder)
      }
      if (form.id) {
        await request({ url: `/menu/items/${form.id}`, method: 'PUT', data: payload })
      } else {
        await request({ url: '/menu/items', method: 'POST', data: payload })
      }
      wx.showToast({ title: '保存成功', icon: 'success' })
      this.closeEditor()
      this.loadData()
    } catch (error) {
      wx.showToast({ title: error.message || '保存失败', icon: 'none' })
    }
  },

  deleteItem(event) {
    const { id } = event.currentTarget.dataset
    wx.showModal({
      title: '删除确认',
      content: '删除后该菜品将不再展示，是否继续？',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        try {
          await request({ url: `/menu/items/${id}`, method: 'DELETE' })
          wx.showToast({ title: '删除成功', icon: 'success' })
          this.loadData()
        } catch (error) {
          wx.showToast({ title: error.message || '删除失败', icon: 'none' })
        }
      }
    })
  }
})
