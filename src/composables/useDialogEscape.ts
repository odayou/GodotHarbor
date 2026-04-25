import { watch, onUnmounted, type Ref } from 'vue'

export function useDialogEscape(dialogRef: Ref<boolean>) {
  let handler: ((e: KeyboardEvent) => void) | null = null

  const stopWatch = watch(dialogRef, (isOpen) => {
    if (isOpen) {
      handler = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          e.preventDefault()
          e.stopPropagation()
          dialogRef.value = false
        }
      }
      document.addEventListener('keydown', handler)
    } else {
      if (handler) {
        document.removeEventListener('keydown', handler)
        handler = null
      }
    }
  }, { immediate: true })

  onUnmounted(() => {
    if (handler) {
      document.removeEventListener('keydown', handler)
      handler = null
    }
    stopWatch()
  })
}
