<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTemplateSigner } from '@/composables/useTemplateSigner'
import { useDialogEscape } from '@/composables/useDialogEscape'
import type { Template } from '@/types'

const {
  keypairs,
  isExporting,
  loadKeypairs,
  exportTemplate,
} = useTemplateSigner()

const props = defineProps<{
  modelValue: boolean
  template: Template | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'exported': [filePath: string]
}>()

useDialogEscape(computed(() => props.modelValue))

const shouldSign = ref(false)
const selectedSignerName = ref('')
const newSignerName = ref('')
const isCreatingKeypair = ref(false)

watch(() => props.modelValue, async (isOpen) => {
  if (isOpen) {
    shouldSign.value = false
    selectedSignerName.value = ''
    newSignerName.value = ''
    await loadKeypairs()
  }
})

const canExport = computed(() => {
  if (isExporting.value) return false
  if (shouldSign.value) {
    return !!selectedSignerName.value || !!newSignerName.value.trim()
  }
  return true
})

const handleCreateKeypair = async () => {
  if (!newSignerName.value.trim()) return
  isCreatingKeypair.value = true
  try {
    const { generateKeypair } = useTemplateSigner()
    const kp = await generateKeypair(newSignerName.value.trim())
    if (kp) {
      selectedSignerName.value = kp.name
      newSignerName.value = ''
    }
  } finally {
    isCreatingKeypair.value = false
  }
}

const handleExport = async () => {
  if (!props.template) return
  const signerName = shouldSign.value
    ? (selectedSignerName.value || newSignerName.value.trim() || undefined)
    : undefined
  const filePath = await exportTemplate(props.template.template_id, props.template.name, signerName)
  if (filePath) {
    emit('exported', filePath)
    emit('update:modelValue', false)
  }
}

const close = () => {
  if (!isExporting.value) {
    emit('update:modelValue', false)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="modelValue && template" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/50" @click="close"></div>
      <div class="dialog-container max-w-md w-full mx-4">
        <div class="p-4">
          <h2 class="text-base font-semibold text-gray-900 dark:text-content-primary mb-3">
            导出模板
          </h2>

          <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer mb-4">
            <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ template.name }}</p>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">
              {{ template.godot.version }} · {{ template.category }} · v{{ template.version }}
            </p>
          </div>

          <div class="space-y-4">
            <div class="flex items-center gap-2">
              <input
                id="export-sign"
                v-model="shouldSign"
                type="checkbox"
                :disabled="isExporting"
                class="w-4 h-4 rounded border-gray-300 dark:border-surface-border text-primary-600 focus:ring-primary-500"
              />
              <label for="export-sign" class="text-sm text-gray-700 dark:text-content-secondary cursor-pointer">
                签名模板（推荐）
              </label>
            </div>

            <div v-if="shouldSign" class="space-y-3 pl-6">
              <div v-if="keypairs.length > 0">
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">选择签名者</label>
                <select
                  v-model="selectedSignerName"
                  :disabled="isExporting"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
                >
                  <option value="">-- 选择已有密钥 --</option>
                  <option v-for="kp in keypairs" :key="kp.public_key" :value="kp.name">
                    {{ kp.name }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">
                  {{ keypairs.length > 0 ? '或创建新签名者' : '创建新签名者' }}
                </label>
                <div class="flex gap-2">
                  <input
                    v-model="newSignerName"
                    type="text"
                    placeholder="签名者名称"
                    :disabled="isExporting || isCreatingKeypair"
                    class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                  />
                  <button
                    @click="handleCreateKeypair"
                    :disabled="isCreatingKeypair || !newSignerName.trim()"
                    class="px-3 py-2 text-sm rounded-lg bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50"
                  >
                    {{ isCreatingKeypair ? '...' : '生成' }}
                  </button>
                </div>
              </div>

              <p class="text-xs text-gray-500 dark:text-content-muted">
                签名可让接收者验证模板来源和完整性，防止模板被篡改。
              </p>
            </div>

            <div v-if="!shouldSign" class="pl-6">
              <div class="p-2.5 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
                <p class="text-xs text-yellow-700 dark:text-yellow-400">
                  未签名的模板在导入时会显示警告提示。建议签名以增强信任度。
                </p>
              </div>
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <button
              @click="close"
              :disabled="isExporting"
              class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
            >
              取消
            </button>
            <button
              @click="handleExport"
              :disabled="!canExport"
              class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
            >
              {{ isExporting ? '导出中...' : '导出' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
