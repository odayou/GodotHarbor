// src/composables/useI18n.ts
// 兼容层，保持旧的 API 接口
import { useI18n as useVueI18n } from 'vue-i18n'

export function useI18n() {
  const { t, locale } = useVueI18n()
  
  return {
    t,
    locale: locale.value,
    setLocale: (lang: string) => {
      locale.value = lang
    }
  }
}