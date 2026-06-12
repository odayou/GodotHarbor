<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { ContextMenuEntry } from '@/composables/useContextMenu'

const props = defineProps<{
  visible: boolean
  x: number
  y: number
  items: ContextMenuEntry[]
}>()

const emit = defineEmits<{
  close: []
}>()

const menuRef = ref<HTMLElement | null>(null)
const adjustedX = ref(props.x)
const adjustedY = ref(props.y)

watch(() => props.visible, async (val) => {
  if (val) {
    adjustedX.value = props.x
    adjustedY.value = props.y
    await nextTick()
    if (menuRef.value) {
      const rect = menuRef.value.getBoundingClientRect()
      const vw = window.innerWidth
      const vh = window.innerHeight
      if (rect.right > vw) {
        adjustedX.value = vw - rect.width - 4
      }
      if (rect.bottom > vh) {
        adjustedY.value = vh - rect.height - 4
      }
      if (adjustedX.value < 0) adjustedX.value = 4
      if (adjustedY.value < 0) adjustedY.value = 4
    }
  }
})
</script>

<template>
  <teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-50" @click="emit('close')" @contextmenu.prevent="emit('close')">
      <div
        ref="menuRef"
        class="fixed bg-white dark:bg-surface-layer border border-border rounded-md py-1 min-w-[180px] text-sm"
        :style="{ left: adjustedX + 'px', top: adjustedY + 'px' }"
        @click.stop
      >
        <template v-for="(item, i) in items" :key="i">
          <div v-if="item.separator" class="my-1 border-t border-border" />
          <button
            v-else
            class="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-primary-50 dark:hover:bg-primary-900/20 disabled:opacity-40 disabled:pointer-events-none"
            :class="{ 'text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20': item.danger }"
            :disabled="item.disabled"
            @click="item.action(); emit('close')"
          >
            <span v-if="item.icon" class="w-4 h-4 flex items-center justify-center text-gray-400 dark:text-content-muted" v-html="item.icon"></span>
            <span class="flex-1 text-gray-700 dark:text-content-primary">{{ item.label }}</span>
            <span v-if="item.shortcut" class="text-xs text-gray-400 dark:text-content-muted ml-4">{{ item.shortcut }}</span>
          </button>
        </template>
      </div>
    </div>
  </teleport>
</template>
