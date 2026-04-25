import { ref } from 'vue'

const isVisible = ref(false)

export function useOnboarding() {
  const showOnboarding = () => {
    isVisible.value = true
  }

  const hideOnboarding = () => {
    isVisible.value = false
  }

  return {
    isVisible,
    showOnboarding,
    hideOnboarding
  }
}
