<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { api } from '@/api'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { open } from '@tauri-apps/plugin-dialog'
import type { TemplateManifest, SignatureVerification } from '@/types'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'imported': []
}>()

useDialogEscape(computed(() => props.modelValue))

const selectedFilePath = ref('')
const manifest = ref<TemplateManifest | null>(null)
const verification = ref<SignatureVerification | null>(null)
const isVerifying = ref(false)
const isConfirming = ref(false)

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    selectedFilePath.value = ''
    manifest.value = null
    verification.value = null
  }
})

const selectFile = async () => {
  const selected = await open({
    multiple: false,
    title: '选择模板文件',
    filters: [{
      name: 'Harbor Template',
      extensions: ['harbor-template']
    }]
  })
  if (selected) {
    selectedFilePath.value = selected as string
    await handleImportAndVerify()
  }
}

const handleImportAndVerify = async () => {
  if (!selectedFilePath.value) return
  isVerifying.value = true
  verification.value = null
  try {
    const result = await api.importTemplateFromFile(selectedFilePath.value)
    if (result) {
      manifest.value = result
      const verifyResult = await api.verifyTemplateSignature(result)
      verification.value = verifyResult
    }
  } catch (e: any) {
    // Verification/read failed - show error
    verification.value = {
      is_valid: false,
      signed_by: null,
      checksum_valid: false,
      error: e?.toString() || '读取模板文件失败',
    }
  } finally {
    isVerifying.value = false
  }
}

const isVerified = computed(() => {
  if (!verification.value) return false
  return verification.value.is_valid && verification.value.checksum_valid
})

const isUnsigned = computed(() => {
  if (!manifest.value) return false
  return !manifest.value.signature
})

const canConfirmImport = computed(() => {
  if (!manifest.value || isConfirming.value) return false
  // Allow import even if unsigned, but verification must have been attempted
  return verification.value !== null
})

const confirmImport = async () => {
  if (!manifest.value) return
  isConfirming.value = true
  try {
    await api.confirmImportTemplate(manifest.value)
    emit('imported')
    emit('update:modelValue', false)
  } catch (e: any) {
    verification.value = {
      is_valid: false,
      signed_by: null,
      checksum_valid: false,
      error: `导入失败: ${e?.toString() || e}`,
    }
  } finally {
    isConfirming.value = false
  }
}

const close = () => {
  if (!isConfirming.value && !isVerifying.value) {
    emit('update:modelValue', false)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/50" @click="close"></div>
      <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-lg w-full mx-4">
        <div class="p-6">
          <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">
            导入模板文件
          </h2>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">选择模板文件</label>
              <div class="flex gap-2">
                <input
                  :value="selectedFilePath"
                  type="text"
                  readonly
                  placeholder="点击选择 .harbor-template 文件"
                  class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary outline-none"
                />
                <button
                  @click="selectFile"
                  :disabled="isVerifying || isConfirming"
                  class="px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
                >
                  浏览
                </button>
              </div>
            </div>

            <!-- Loading -->
            <div v-if="isVerifying" class="flex items-center justify-center py-6">
              <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
              <span class="ml-3 text-sm text-gray-500 dark:text-content-muted">正在验证模板...</span>
            </div>

            <!-- Verification Result -->
            <div v-if="verification && manifest" class="space-y-3">
              <!-- Template Info -->
              <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer">
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ manifest.template.name }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">
                  {{ manifest.template.godot.version }} · {{ manifest.template.category }} · v{{ manifest.template.version }}
                </p>
              </div>

              <!-- Signed & Verified -->
              <div v-if="isVerified" class="p-3 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800">
                <div class="flex items-center gap-2">
                  <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                  <div>
                    <p class="text-sm font-medium text-green-700 dark:text-green-400">签名验证通过</p>
                    <p class="text-xs text-green-600 dark:text-green-500">签名者: {{ verification.signed_by }}</p>
                  </div>
                </div>
              </div>

              <!-- Unsigned Warning -->
              <div v-else-if="isUnsigned" class="p-3 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
                <div class="flex items-center gap-2">
                  <svg class="w-5 h-5 text-yellow-600 dark:text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  <div>
                    <p class="text-sm font-medium text-yellow-700 dark:text-yellow-400">未签名模板</p>
                    <p class="text-xs text-yellow-600 dark:text-yellow-500">此模板未经过签名，无法验证来源。导入时请确保来源可信。</p>
                  </div>
                </div>
              </div>

              <!-- Verification Failed -->
              <div v-else class="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
                <div class="flex items-center gap-2">
                  <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <div>
                    <p class="text-sm font-medium text-red-700 dark:text-red-400">验证失败</p>
                    <p class="text-xs text-red-600 dark:text-red-500">{{ verification.error }}</p>
                  </div>
                </div>
              </div>

              <!-- Checksum Info -->
              <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-content-muted">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4" />
                </svg>
                <span>校验和: {{ manifest.checksum.substring(0, 16) }}...</span>
                <span :class="verification.checksum_valid ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
                  {{ verification.checksum_valid ? '匹配' : '不匹配' }}
                </span>
              </div>
            </div>
          </div>

          <div class="flex gap-3 mt-6">
            <button
              @click="close"
              :disabled="isConfirming || isVerifying"
              class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
            >
              取消
            </button>
            <button
              v-if="manifest"
              @click="confirmImport"
              :disabled="!canConfirmImport"
              class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
            >
              {{ isConfirming ? '导入中...' : '确认导入' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
