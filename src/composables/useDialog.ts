import { ref, onMounted, onUnmounted } from 'vue'

export function useDialog() {
  const isOpen = ref(false)
  let escapeHandler: ((e: KeyboardEvent) => void) | null = null

  const openDialog = () => {
    isOpen.value = true
    escapeHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        closeDialog()
      }
    }
    document.addEventListener('keydown', escapeHandler)
  }

  const closeDialog = () => {
    isOpen.value = false
    if (escapeHandler) {
      document.removeEventListener('keydown', escapeHandler)
      escapeHandler = null
    }
  }

  const onOverlayClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      closeDialog()
    }
  }

  onUnmounted(() => {
    if (escapeHandler) {
      document.removeEventListener('keydown', escapeHandler)
      escapeHandler = null
    }
  })

  return {
    isOpen,
    openDialog,
    closeDialog,
    onOverlayClick
  }
}
