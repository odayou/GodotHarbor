<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { getCurrentWindow } from '@tauri-apps/api/window'

const route = useRoute()
const { t } = useI18n()
const appWindow = getCurrentWindow()

const pageTitle = computed(() => {
  const name = route.name as string
  if (!name) return ''
  const key = `nav.${name}`
  const translated = t(key)
  return translated === key ? '' : translated
})

const minimize = async () => {
  try { await appWindow.minimize() } catch (e) { console.error('minimize failed:', e) }
}
const toggleMaximize = async () => {
  try { await appWindow.toggleMaximize() } catch (e) { console.error('toggleMaximize failed:', e) }
}
const close = async () => {
  try { await appWindow.close() } catch (e) { console.error('close failed:', e) }
}
</script>

<template>
  <div class="flex items-center h-10 header-acrylic shadow-sm border-b border-gray-200/60 dark:border-surface-border/40 select-none" data-tauri-drag-region>
    <!-- Left: Logo + App name + Page title -->
    <div class="flex items-center gap-2 pl-3 flex-1 min-w-0" data-tauri-drag-region>
      <img src="/favicon.png" alt="Harbor" class="w-4 h-4 shrink-0" />
      <span class="text-xs font-semibold text-primary-600 dark:text-brand-primary whitespace-nowrap">Harbor</span>
      <span v-if="pageTitle" class="text-xs text-gray-400 dark:text-content-muted whitespace-nowrap">/ {{ pageTitle }}</span>
    </div>

    <!-- Right: Window controls -->
    <div class="flex h-full shrink-0">
      <button
        @click="minimize"
        class="w-11 h-full flex items-center justify-center text-gray-500 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-white/10 transition-colors"
        aria-label="最小化"
      >
        <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor"><rect width="10" height="1"/></svg>
      </button>
      <button
        @click="toggleMaximize"
        class="w-11 h-full flex items-center justify-center text-gray-500 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-white/10 transition-colors"
        aria-label="最大化"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1"><rect x="0.5" y="0.5" width="9" height="9"/></svg>
      </button>
      <button
        @click="close"
        class="w-11 h-full flex items-center justify-center text-gray-500 dark:text-content-secondary hover:bg-red-500 hover:text-white transition-colors"
        aria-label="关闭"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2"><line x1="0" y1="0" x2="10" y2="10"/><line x1="10" y1="0" x2="0" y2="10"/></svg>
      </button>
    </div>
  </div>
</template>
