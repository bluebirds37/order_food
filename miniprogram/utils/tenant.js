function parseTenantCode(raw) {
  if (!raw) {
    return ''
  }
  const text = String(raw).trim()
  if (!text) {
    return ''
  }
  const matched = text.match(/tenant[/=:]([A-Za-z0-9_-]+)/i)
  if (matched) {
    return matched[1].toUpperCase()
  }
  if (/^[A-Za-z0-9_-]{2,32}$/.test(text)) {
    return text.toUpperCase()
  }
  return ''
}

module.exports = {
  parseTenantCode
}
