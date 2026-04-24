<script setup lang="ts">
import { computed } from 'vue'
import { useTheme } from '@/composables/useTheme'

const { currentTheme, setTheme } = useTheme()

const isDarkMode = computed(() => {
  return currentTheme.value === 'dark' ||
    (currentTheme.value === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
})

const toggleDarkMode = () => {
  if (currentTheme.value === 'dark') {
    setTheme('light')
  } else {
    setTheme('dark')
  }
}
</script>

<template>
  <header class="h-16 bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 flex items-center justify-between px-6">
    <div class="flex items-center space-x-4">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
        Godot 开发环境管理中枢
      </h2>
    </div>
    
    <div class="flex items-center space-x-4">
      <button
        @click="toggleDarkMode"
        class="p-2 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
      >
        <svg v-if="isDarkMode" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
        <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      </button>
    </div>
  </header>
</template>
