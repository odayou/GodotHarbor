<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTemplateSigner } from '@/composables/useTemplateSigner'
import { useDialogEscape } from '@/composables/useDialogEscape'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const { t } = useI18n()

const {
  keypairs,
  isLoadingKeypairs,
  isGeneratingKeypair,
  loadKeypairs,
  generateKeypair,
  deleteKeypair,
} = useTemplateSigner()

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

useDialogEscape(computed(() => props.modelValue))

const newKeypairName = ref('')
const showDeleteConfirm = ref(false)
const deleteTargetPublicKey = ref('')
const expandedPublicKey = ref('')

watch(() => props.modelValue, async (isOpen) => {
  if (isOpen) {
    newKeypairName.value = ''
    expandedPublicKey.value = ''
    await loadKeypairs()
  }
})

const handleGenerate = async () => {
  if (!newKeypairName.value.trim()) return
  await generateKeypair(newKeypairName.value.trim())
  newKeypairName.value = ''
}

const handleDelete = async () => {
  if (!deleteTargetPublicKey.value) return
  await deleteKeypair(deleteTargetPublicKey.value)
  showDeleteConfirm.value = false
  deleteTargetPublicKey.value = ''
}

const toggleExpand = (publicKey: string) => {
  expandedPublicKey.value = expandedPublicKey.value === publicKey ? '' : publicKey
}

const close = () => {
  emit('update:modelValue', false)
}
</script>

<template>
  <Teleport to="body">
    <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/50" @click="close"></div>
      <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-lg w-full mx-4 max-h-[80vh] flex flex-col">
        <div class="p-6 pb-4 border-b border-gray-200 dark:border-surface-border">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary">{{ t('keypair.title') }}</h2>
            <button @click="close" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-500">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <p class="text-sm text-gray-500 dark:text-content-muted mt-1">
            {{ t('keypair.description') }}
          </p>
        </div>

        <div class="flex-1 overflow-y-auto p-6 space-y-4">
          <div class="p-4 rounded-lg bg-gray-50 dark:bg-surface-layer">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('keypair.generateNew') }}</h3>
            <div class="flex gap-2">
              <input
                v-model="newKeypairName"
                type="text"
                :placeholder="t('keypair.signerNamePlaceholder')"
                :disabled="isGeneratingKeypair"
                class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                @keyup.enter="handleGenerate"
              />
              <button
                @click="handleGenerate"
                :disabled="isGeneratingKeypair || !newKeypairName.trim()"
                class="px-4 py-2 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50"
              >
                {{ isGeneratingKeypair ? '...' : t('common.generate') }}
              </button>
            </div>
          </div>

          <div v-if="isLoadingKeypairs" class="flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
          </div>

          <div v-else-if="keypairs.length === 0" class="text-center py-8">
            <svg class="w-12 h-12 mx-auto text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
            </svg>
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('keypair.noKeypairs') }}</p>
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="kp in keypairs"
              :key="kp.public_key"
              class="p-3 rounded-lg border border-gray-200 dark:border-surface-border"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2 min-w-0">
                  <svg class="w-4 h-4 text-primary-600 dark:text-brand-primary flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                  <span class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ kp.name }}</span>
                  <span class="text-xs text-gray-400 dark:text-content-muted flex-shrink-0">{{ kp.created_at.split('T')[0] }}</span>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <button
                    @click="toggleExpand(kp.public_key)"
                    class="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-400 hover:text-gray-600 dark:hover:text-content-secondary"
                    :title="t('keypair.viewPublicKey')"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  </button>
                  <button
                    @click="deleteTargetPublicKey = kp.public_key; showDeleteConfirm = true"
                    class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 text-gray-400 hover:text-red-600 dark:hover:text-red-400"
                    :title="t('common.delete')"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
              <div v-if="expandedPublicKey === kp.public_key" class="mt-2">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('keypair.publicKey') }}:</p>
                <div class="p-2 rounded bg-gray-100 dark:bg-surface-hover text-xs font-mono text-gray-700 dark:text-content-secondary break-all select-all">
                  {{ kp.public_key }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('keypair.deleteKeypair')"
      :description="t('keypair.deleteKeypairConfirm')"
      :confirm-text="t('common.delete')"
      confirm-color="red"
      @confirm="handleDelete"
    />
  </Teleport>
</template>
