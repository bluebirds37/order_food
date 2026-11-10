const { request, uploadImage } = require('../../utils/request')
const { requireAdmin, tenantQuery } = require('../../utils/admin')

function createDefaultForm() {
  return {
    id: null,
    categoryId: 1,
    name: '',
    price: '0.00',
    imageUrl: '',
    detailImages: [],
    optionsText: '',
    description: '',
    sortOrder: 1,
    recommended: false,
    available: true
  }
}

Page({
  data: {
    tenantCode: '',
    loading: false,
    menuItems: [],
    categories: [],
    showEditor: false,
    showCategories: false,
    uploading: false,
    currentCategoryName: '',
    form: createDefaultForm()
  },

  onShow() {
    const session = requireAdmin()
    if (!session) {
      return
    }
    this.setData({ tenantCode: session.tenantCode })
    this.loadData(session.tenantCode)
  },

  async loadData(tenantCode) {
    this.setData({ loading: true })
    try {
      const overview = await request({
        url: `/menu/overview${tenantQuery(tenantCode)}`,
        method: 'GET'
      })
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
    if (!this.data.categories.length) {
      wx.showToast({ title: '请先创建菜品分类', icon: 'none' })
      return
    }
    const defaultCategoryId = this.data.categories[0].id
    this.setData({
      showEditor: true,
      currentCategoryName: this.data.categories[0].name,
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
        detailImages: target.detailImages || [],
        optionsText: (target.options || []).join('、'),
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
    const { form, tenantCode } = this.data
    if (!form.name || !form.name.trim()) {
      wx.showToast({ title: '请填写菜品名称', icon: 'none' })
      return
    }
    if (Number(form.price) < 0) {
      wx.showToast({ title: '价格不能为负数', icon: 'none' })
      return
    }
    try {
      const payload = {
        ...form,
        price: Number(form.price),
        sortOrder: Number(form.sortOrder),
        options: (form.optionsText || '')
          .split(/[、，,]/)
          .map((value) => value.trim())
          .filter(Boolean)
      }
      if (form.id) {
        await request({
          url: `/menu/items/${form.id}${tenantQuery(tenantCode)}`,
          method: 'PUT',
          data: payload
        })
      } else {
        await request({
          url: `/menu/items${tenantQuery(tenantCode)}`,
          method: 'POST',
          data: payload
        })
      }
      wx.showToast({ title: '保存成功', icon: 'success' })
      this.closeEditor()
      this.loadData(tenantCode)
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
          await request({
            url: `/menu/items/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'DELETE'
          })
          wx.showToast({ title: '删除成功', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '删除失败', icon: 'none' })
        }
      }
    })
  },

  toggleCategories() {
    this.setData({ showCategories: !this.data.showCategories })
  },

  addCategory() {
    wx.showModal({
      title: '新增分类',
      editable: true,
      placeholderText: '请输入分类名称',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        const name = (result.content || '').trim()
        if (!name) {
          wx.showToast({ title: '分类名称不能为空', icon: 'none' })
          return
        }
        try {
          await request({
            url: `/menu/categories${tenantQuery(this.data.tenantCode)}`,
            method: 'POST',
            data: { name }
          })
          wx.showToast({ title: '分类已新增', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '新增失败', icon: 'none' })
        }
      }
    })
  },

  editCategory(event) {
    const { id } = event.currentTarget.dataset
    const category = this.data.categories.find((item) => item.id === id)
    if (!category) {
      return
    }
    wx.showModal({
      title: '重命名分类',
      editable: true,
      content: category.name,
      placeholderText: '请输入分类名称',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        const name = (result.content || '').trim()
        if (!name || name === category.name) {
          return
        }
        try {
          await request({
            url: `/menu/categories/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'PUT',
            data: { name, sortOrder: category.sortOrder }
          })
          wx.showToast({ title: '分类已更新', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '更新失败', icon: 'none' })
        }
      }
    })
  },

  deleteCategory(event) {
    const { id } = event.currentTarget.dataset
    wx.showModal({
      title: '删除分类',
      content: '分类下仍有菜品时无法删除，是否继续？',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        try {
          await request({
            url: `/menu/categories/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'DELETE'
          })
          wx.showToast({ title: '分类已删除', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '删除失败', icon: 'none' })
        }
      }
    })
  },

  chooseImage() {
    wx.chooseMedia({
      count: 1,
      mediaType: ['image'],
      success: async (result) => {
        const file = result.tempFiles && result.tempFiles[0]
        if (!file) {
          return
        }
        this.setData({ uploading: true })
        try {
          const image = await uploadImage(file.tempFilePath)
          this.setData({ 'form.imageUrl': image.url })
          wx.showToast({ title: '图片已上传', icon: 'success' })
        } catch (error) {
          wx.showToast({ title: error.message || '上传失败', icon: 'none' })
        } finally {
          this.setData({ uploading: false })
        }
      }
    })
  },

  removeImage() {
    this.setData({ 'form.imageUrl': '' })
  },

  addDetailImages() {
    const remain = 9 - this.data.form.detailImages.length
    if (remain <= 0) {
      wx.showToast({ title: '详情图片最多9张', icon: 'none' })
      return
    }
    wx.chooseMedia({
      count: remain,
      mediaType: ['image'],
      success: async (result) => {
        const files = (result.tempFiles || []).map((file) => file.tempFilePath).filter(Boolean)
        if (!files.length) {
          return
        }
        this.setData({ uploading: true })
        try {
          const urls = []
          for (const filePath of files) {
            const image = await uploadImage(filePath)
            urls.push(image.url)
          }
          this.setData({ 'form.detailImages': [...this.data.form.detailImages, ...urls] })
          wx.showToast({ title: '详情图片已上传', icon: 'success' })
        } catch (error) {
          wx.showToast({ title: error.message || '上传失败', icon: 'none' })
        } finally {
          this.setData({ uploading: false })
        }
      }
    })
  },

  removeDetailImage(event) {
    const { index } = event.currentTarget.dataset
    const detailImages = this.data.form.detailImages.filter((_, i) => i !== index)
    this.setData({ 'form.detailImages': detailImages })
  },

  async toggleAvailable(event) {
    const { id } = event.currentTarget.dataset
    const target = this.data.menuItems.find((item) => item.id === id)
    if (!target) {
      return
    }
    try {
      await request({
        url: `/menu/items/${id}${tenantQuery(this.data.tenantCode)}`,
        method: 'PUT',
        data: {
          categoryId: target.categoryId,
          name: target.name,
          price: Number(target.price),
          description: target.description,
          imageUrl: target.imageUrl,
          detailImages: target.detailImages || [],
          recommended: target.recommended,
          available: !target.available,
          sortOrder: target.sortOrder
        }
      })
      wx.showToast({ title: target.available ? '已下架' : '已上架', icon: 'success' })
      this.loadData(this.data.tenantCode)
    } catch (error) {
      wx.showToast({ title: error.message || '操作失败', icon: 'none' })
    }
  },

  editPrice(event) {
    const { id } = event.currentTarget.dataset
    const target = this.data.menuItems.find((item) => item.id === id)
    if (!target) {
      return
    }
    wx.showModal({
      title: '修改价格',
      editable: true,
      content: String(target.price),
      placeholderText: '请输入新价格',
      success: async (result) => {
        if (!result.confirm) {
          return
        }
        const price = Number(result.content)
        if (!Number.isFinite(price) || price < 0) {
          wx.showToast({ title: '价格不合法', icon: 'none' })
          return
        }
        try {
          await request({
            url: `/menu/items/${id}${tenantQuery(this.data.tenantCode)}`,
            method: 'PUT',
            data: {
              categoryId: target.categoryId,
              name: target.name,
              price,
              description: target.description,
              imageUrl: target.imageUrl,
              detailImages: target.detailImages || [],
              recommended: target.recommended,
              available: target.available,
              sortOrder: target.sortOrder
            }
          })
          wx.showToast({ title: '价格已更新', icon: 'success' })
          this.loadData(this.data.tenantCode)
        } catch (error) {
          wx.showToast({ title: error.message || '改价失败', icon: 'none' })
        }
      }
    })
  },

  previewMenu() {
    wx.navigateTo({
      url: `/pages/menu/index?preview=1&tenantCode=${encodeURIComponent(this.data.tenantCode)}`
    })
  }
})
