// src/locales/index.ts
import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import en from './en'

// 从localStorage读取语言设置，默认使用浏览器语言
const savedLanguage = localStorage.getItem('godotharbor-language')
const browserLanguage = navigator.language

// 检测浏览器语言，优先使用中文或英文
let initialLocale = savedLanguage || 'zh-CN'
if (!savedLanguage) {
  if (browserLanguage.startsWith('en')) {
    initialLocale = 'en'
  } else if (browserLanguage.startsWith('zh')) {
    initialLocale = 'zh-CN'
  }
}

export default createI18n({
  legacy: false, // 使用 Composition API
  locale: initialLocale,
  fallbackLocale: 'en', // 默认为英文
  messages: {
    'zh-CN': zhCN,
    en: en
  }
})