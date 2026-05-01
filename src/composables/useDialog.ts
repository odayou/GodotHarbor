import { ref } from 'vue'
import { useDialogEscape } from '@/composables/useDialogEscape'

export function useDialog() {
  const isOpen = ref(false)

  useDialogEscape(isOpen)

  const openDialog = () => {
    isOpen.value = true
  }

  const closeDialog = () => {
    isOpen.value = false
  }

  const onOverlayClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      closeDialog()
    }
  }

  return {
    isOpen,
    openDialog,
    closeDialog,
    onOverlayClick
  }
}
