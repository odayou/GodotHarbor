import { ref, computed, type Ref } from 'vue'

export function useBatchSelection<T>(options: {
  items: Ref<T[]>
  getId: (item: T) => string
}) {
  const selectedIds = ref<Set<string>>(new Set())
  const lastClickedIndex = ref<number>(-1)
  const isBatchMode = ref(false)
  const selectedCount = computed(() => selectedIds.value.size)

  const toggleSelection = (item: T, event: MouseEvent | Event) => {
    const mouseEvent = event as MouseEvent
    const id = options.getId(item)
    const currentList = options.items.value
    const currentIndex = currentList.findIndex(i => options.getId(i) === id)

    if (mouseEvent.shiftKey && lastClickedIndex.value >= 0) {
      const start = Math.min(lastClickedIndex.value, currentIndex)
      const end = Math.max(lastClickedIndex.value, currentIndex)
      for (let i = start; i <= end; i++) {
        selectedIds.value.add(options.getId(currentList[i]))
      }
    } else if (mouseEvent.ctrlKey || mouseEvent.metaKey) {
      if (selectedIds.value.has(id)) {
        selectedIds.value.delete(id)
      } else {
        selectedIds.value.add(id)
      }
    } else {
      if (selectedIds.value.has(id)) {
        selectedIds.value.delete(id)
        if (selectedIds.value.size === 0) {
          isBatchMode.value = false
        }
      } else {
        selectedIds.value.add(id)
        isBatchMode.value = true
      }
    }

    lastClickedIndex.value = currentIndex
    selectedIds.value = new Set(selectedIds.value)
  }

  const selectAll = () => {
    for (const item of options.items.value) {
      selectedIds.value.add(options.getId(item))
    }
    selectedIds.value = new Set(selectedIds.value)
    isBatchMode.value = true
  }

  const clearSelection = () => {
    selectedIds.value.clear()
    selectedIds.value = new Set(selectedIds.value)
    isBatchMode.value = false
    lastClickedIndex.value = -1
  }

  return {
    selectedIds,
    lastClickedIndex,
    isBatchMode,
    selectedCount,
    toggleSelection,
    selectAll,
    clearSelection,
  }
}
