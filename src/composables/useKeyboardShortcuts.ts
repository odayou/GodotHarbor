import { onMounted, onUnmounted } from 'vue'

export interface KeyboardShortcut {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  handler: () => void
  description: string
  global?: boolean
}

const shortcuts: KeyboardShortcut[] = []

export function useKeyboardShortcuts() {
  function registerShortcut(shortcut: KeyboardShortcut) {
    shortcuts.push(shortcut)
  }

  function unregisterShortcut(key: string, ctrl?: boolean, shift?: boolean, alt?: boolean) {
    const index = shortcuts.findIndex(s =>
      s.key.toLowerCase() === key.toLowerCase() &&
      (ctrl ? s.ctrl : !s.ctrl) &&
      (shift ? s.shift : !s.shift) &&
      (alt ? s.alt : !s.alt)
    )
    if (index > -1) {
      shortcuts.splice(index, 1)
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    const target = event.target as HTMLElement
    const isInInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT'

    for (const shortcut of shortcuts) {
      if (isInInput && !shortcut.global) continue

      const keyMatch = shortcut.key.toLowerCase() === event.key.toLowerCase()
      const ctrlMatch = shortcut.ctrl ? (event.ctrlKey || event.metaKey) : !(event.ctrlKey || event.metaKey)
      const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey
      const altMatch = shortcut.alt ? event.altKey : !event.altKey

      if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
        event.preventDefault()
        shortcut.handler()
        return
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })

  return {
    registerShortcut,
    unregisterShortcut,
    shortcuts
  }
}