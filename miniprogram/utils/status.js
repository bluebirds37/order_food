const ORDER_STATUS_TEXT = {
  PENDING_PAYMENT: '待支付',
  PAID: '已支付',
  PREPARING: '制作中',
  COMPLETED: '已完成',
  CANCELLED: '已取消'
}

function orderStatusText(status) {
  return ORDER_STATUS_TEXT[status] || '未知状态'
}

function withStatusText(list) {
  return (list || []).map((item) => ({
    ...item,
    statusText: orderStatusText(item.status)
  }))
}

module.exports = {
  ORDER_STATUS_TEXT,
  orderStatusText,
  withStatusText
}
