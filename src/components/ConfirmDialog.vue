<script setup lang="ts">
import { computed, watch, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  confirmText?: string
  confirmColor?: string
  cancelText?: string
  modelValue: boolean
}>(), {
  title: '确认操作',
  description: '',
  confirmText: '确认',
  confirmColor: 'red',
  cancelText: '取消'
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'confirm': []
}>()

let escapeHandler: ((e: KeyboardEvent) => void) | null = null

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    escapeHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        e.stopPropagation()
        emit('update:modelValue', false)
      }
    }
    document.addEventListener('keydown', escapeHandler)
  } else {
    if (escapeHandler) {
      document.removeEventListener('keydown', escapeHandler)
      escapeHandler = null
    }
  }
}, { immediate: true })

onUnmounted(() => {
  if (escapeHandler) {
    document.removeEventListener('keydown', escapeHandler)
    escapeHandler = null
  }
})

const close = () => {
  emit('update:modelValue', false)
}

const onOverlayClick = (e: MouseEvent) => {
  if (e.target === e.currentTarget) {
    close()
  }
}

const onConfirm = () => {
  emit('confirm')
  close()
}

const confirmColorClass = computed(() => {
  const map: Record<string, string> = {
    red: 'bg-red-600 hover:bg-red-700',
    orange: 'bg-orange-600 hover:bg-orange-700',
    yellow: 'bg-yellow-600 hover:bg-yellow-700',
    blue: 'bg-blue-600 hover:bg-blue-700',
    primary: 'bg-primary-600 hover:bg-primary-700',
  }
  return map[props.confirmColor] || map.red
})
</script>

<template>
  <div
    v-if="modelValue"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click="onOverlayClick"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
        {{ title }}
      </h3>
      <p v-if="description" class="text-sm text-gray-600 dark:text-gray-400 mb-6">
        {{ description }}
      </p>
      <div class="flex justify-end space-x-3">
        <button
          @click="close"
          class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
        >
          {{ cancelText }}
        </button>
        <button
          @click="onConfirm"
          :class="['px-4 py-2 text-white rounded-lg', confirmColorClass]"
        >
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>
