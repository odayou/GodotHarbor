<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { useSidebar } from '@/composables/useSidebar'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const { isCollapsed, toggleSidebar } = useSidebar()

const asideClass = computed(() => {
  return [
    'sidebar-acrylic shadow-lg flex flex-col shrink-0 transition-all duration-200 ease-in-out border-r border-gray-200/50 dark:border-surface-border',
    isCollapsed.value ? 'w-16' : 'w-54'
  ]
})

const menuItems = [
  { path: '/', icon: 'home', labelKey: 'nav.home' },
  { path: '/projects', icon: 'folder', labelKey: 'nav.projects' },
  { path: '/plugins', icon: 'puzzle', labelKey: 'nav.plugins' },
  { path: '/engines', icon: 'engine', labelKey: 'nav.engines' },
  { path: '/updates', icon: 'updates', labelKey: 'nav.updates' },
  { path: '/settings', icon: 'settings', labelKey: 'nav.settings' },
  { path: '/about', icon: 'about', labelKey: 'nav.about' }
]

const navigateTo = (path: string) => {
  router.push(path)
}
</script>

<template>
  <aside :class="asideClass">
    <div
      :class="[
        'h-12 border-b border-gray-200 dark:border-surface-border flex items-center',
        isCollapsed ? 'justify-center px-0' : 'justify-center gap-2 px-2'
      ]"
    >
      <img src="/favicon.png" alt="Godot Harbor" class="w-8 h-8 shrink-0 rounded-lg" />
      <h1
        v-if="!isCollapsed"
        class="text-lg font-bold text-primary-600 dark:text-primary-400 whitespace-nowrap overflow-hidden"
      >
        Godot Harbor
      </h1>
    </div>

    <nav class="flex-1 py-4 overflow-hidden">
      <ul class="space-y-1" :class="isCollapsed ? 'px-1' : 'px-3'">
        <li v-for="item in menuItems" :key="item.path">
          <button
            @click="navigateTo(item.path)"
            :class="[
              'w-full flex items-center rounded-lg transition-colors',
              isCollapsed
                ? 'justify-center p-3'
                : 'px-4 py-3 text-sm font-medium',
              route.path === item.path
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                : 'text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer'
            ]"
            :title="isCollapsed ? t(item.labelKey) : undefined"
          >
            <svg v-if="item.icon === 'home'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
            </svg>
            <svg v-else-if="item.icon === 'folder'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <svg v-else-if="item.icon === 'puzzle'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
            <svg v-else-if="item.icon === 'engine'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <svg v-else-if="item.icon === 'updates'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <svg v-else-if="item.icon === 'about'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else-if="item.icon === 'settings'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <span
              v-if="!isCollapsed"
              class="ml-3 whitespace-nowrap overflow-hidden"
            >
              {{ t(item.labelKey) }}
            </span>
          </button>
        </li>
      </ul>
    </nav>

    <div class="border-t border-gray-200 dark:border-surface-border p-2">
      <button
        @click="toggleSidebar"
        :class="[
          'w-full flex items-center rounded-lg transition-colors',
          isCollapsed ? 'justify-center p-2' : 'px-4 py-2 text-sm text-gray-500 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-surface-layer',
        ]"
        :title="isCollapsed ? t('sidebar.expand') : t('sidebar.collapse')"
      >
        <svg
          class="w-5 h-5 shrink-0 transition-transform duration-200"
          :class="isCollapsed ? 'rotate-180' : ''"
          fill="none" stroke="currentColor" viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        </svg>
        <span v-if="!isCollapsed" class="ml-2">{{ t('sidebar.collapse') }}</span>
      </button>
    </div>
  </aside>
</template>
