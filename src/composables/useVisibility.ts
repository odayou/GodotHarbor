import { ref } from 'vue'

const visibilityMap = new Map<string, ReturnType<typeof ref<boolean>>>()

function getRef(key: string) {
  if (!visibilityMap.has(key)) {
    visibilityMap.set(key, ref(false))
  }
  return visibilityMap.get(key)!
}

export function useVisibility(key: string) {
  const isVisible = getRef(key)
  const show = () => { isVisible.value = true }
  const hide = () => { isVisible.value = false }
  return { isVisible, show, hide }
}
