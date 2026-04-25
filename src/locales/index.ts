// src/locales/index.ts
import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import en from './en'

export default createI18n({
  legacy: false, // 使用 Composition API
  locale: 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    en: en
  }
})