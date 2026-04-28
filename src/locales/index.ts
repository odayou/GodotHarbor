import { createI18n } from 'vue-i18n'
import { compile, resolveValue, fallbackWithLocaleChain } from '@intlify/core-base'
import zhCN from './zh-CN'
import en from './en'

const savedLanguage = localStorage.getItem('godotharbor-language')
const browserLanguage = navigator.language

let initialLocale = savedLanguage || 'zh-CN'
if (!savedLanguage) {
  if (browserLanguage.startsWith('en')) {
    initialLocale = 'en'
  } else if (browserLanguage.startsWith('zh')) {
    initialLocale = 'zh-CN'
  }
}

export default createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en',
  messageCompiler: compile,
  messageResolver: resolveValue,
  localeFallbacker: fallbackWithLocaleChain,
  messages: {
    'zh-CN': zhCN,
    en: en
  }
})
