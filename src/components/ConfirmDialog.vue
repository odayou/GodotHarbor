<script setup lang="ts">
import { computed, watch, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  confirmText?: string
  confirmColor?: string
  cancelText?: string
  modelValue: boolean
}>(), {
  title: '',
  description: '',
  confirmText: '',
  confirmColor: 'red',
  cancelText: ''
})

const resolvedTitle = computed(() => props.title || t('common.confirm'))
const resolvedConfirmText = computed(() => props.confirmText || t('common.confirm'))
const resolvedCancelText = computed(() => props.cancelText || t('common.cancel'))

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
  <Teleport to="body">
  <div
    v-if="modelValue"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click="onOverlayClick"
  >
    <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
      <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-2">
        {{ resolvedTitle }}
      </h3>
      <p v-if="description" class="text-sm text-gray-600 dark:text-content-muted mb-4">
        {{ description }}
      </p>
      <slot></slot>
      <div class="flex justify-end space-x-3 mt-6">
        <button
          @click="close"
          class="btn-secondary"
        >
          {{ resolvedCancelText }}
        </button>
        <button
          @click="onConfirm"
          :class="['px-4 py-2 text-white rounded-lg', confirmColorClass]"
        >
          {{ resolvedConfirmText }}
        </button>
      </div>
    </div>
  </div>
  </Teleport>
</template>
