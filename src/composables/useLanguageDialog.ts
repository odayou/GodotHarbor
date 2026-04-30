import { ref } from 'vue'

const isVisible = ref(false)

export function useLanguageDialog() {
  const showLanguageDialog = () => {
    isVisible.value = true
  }

  const hideLanguageDialog = () => {
    isVisible.value = false
  }

  return {
    isVisible,
    showLanguageDialog,
    hideLanguageDialog
  }
}
