import { ref, onMounted, onUnmounted } from 'vue'

export interface ContextMenuItem {
  label: string
  icon?: string
  action: () => void
  disabled?: boolean
  separator?: false
  shortcut?: string
  danger?: boolean
}

export interface ContextMenuSeparator {
  separator: true
}

export type ContextMenuEntry = ContextMenuItem | ContextMenuSeparator

export function useContextMenu() {
  const visible = ref(false)
  const x = ref(0)
  const y = ref(0)
  const items = ref<ContextMenuEntry[]>([])

  const show = (event: MouseEvent, menuItems: ContextMenuEntry[]) => {
    event.preventDefault()
    items.value = menuItems
    x.value = event.clientX
    y.value = event.clientY
    visible.value = true
  }

  const close = () => {
    visible.value = false
  }

  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && visible.value) {
      close()
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return { visible, x, y, items, show, close }
}
