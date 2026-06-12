<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { useSidebar } from '@/composables/useSidebar'
import { useTheme } from '@/composables/useTheme'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const { isCollapsed, toggleSidebar } = useSidebar()
const { currentTheme, setTheme } = useTheme()

const isDarkMode = computed(() => {
  return currentTheme.value === 'dark' ||
    (currentTheme.value === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
})

const toggleDarkMode = () => {
  if (isDarkMode.value) {
    setTheme('light')
  } else {
    setTheme('dark')
  }
}

const asideClass = computed(() => {
  return [
    'sidebar-acrylic shadow-md flex flex-col shrink-0 transition-all duration-200 ease-in-out border-r border-gray-200/60 dark:border-surface-border/40',
    isCollapsed.value ? 'w-14' : 'w-54'
  ]
})

const menuItems = [
  { path: '/', icon: 'home', labelKey: 'nav.home' },
  { path: '/projects', icon: 'folder', labelKey: 'nav.projects' },
  { path: '/plugins', icon: 'puzzle', labelKey: 'nav.plugins' },
  { path: '/engines', icon: 'engine', labelKey: 'nav.engines' },
  { path: '/build', icon: 'build', labelKey: 'nav.build' },
  { path: '/updates', icon: 'updates', labelKey: 'nav.updates' },
]

const bottomItems = [
  { path: '/settings', icon: 'settings', labelKey: 'nav.settings' },
  { path: '/about', icon: 'about', labelKey: 'nav.about' },
]

const navigateTo = (path: string) => {
  router.push(path)
}
</script>

<template>
  <aside :class="asideClass">
    <!-- Logo area -->
    <div
      :class="[
        'h-10 border-b border-gray-200/60 dark:border-surface-border/40 flex items-center',
        isCollapsed ? 'justify-center px-0' : 'gap-2 px-3'
      ]"
    >
      <img src="/favicon.png" alt="Harbor" class="w-6 h-6 shrink-0" />
      <span
        v-if="!isCollapsed"
        class="text-sm font-semibold text-primary-600 dark:text-brand-primary whitespace-nowrap overflow-hidden"
      >
        Harbor
      </span>
    </div>

    <!-- Main nav -->
    <nav class="flex-1 py-2 overflow-hidden">
      <ul class="space-y-1" :class="isCollapsed ? 'px-1' : 'px-2'">
        <li v-for="item in menuItems" :key="item.path">
          <button
            @click="navigateTo(item.path)"
            :class="[
              'w-full flex items-center rounded-[4px] transition-colors duration-100',
              isCollapsed
                ? 'justify-center p-2'
                : 'px-3 py-2 text-[13px]',
              route.path === item.path
                ? 'bg-primary-50/60 dark:bg-primary-500/10 text-primary-600 dark:text-brand-primary'
                : 'text-gray-600 dark:text-content-secondary hover:bg-black/[0.04] dark:hover:bg-white/[0.06]'
            ]"
            :title="isCollapsed ? t(item.labelKey) : undefined"
          >
            <svg v-if="item.icon === 'home'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
            </svg>
            <svg v-else-if="item.icon === 'folder'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <svg v-else-if="item.icon === 'puzzle'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
            <svg v-else-if="item.icon === 'engine'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <svg v-else-if="item.icon === 'build'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            <svg v-else-if="item.icon === 'updates'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span
              v-if="!isCollapsed"
              class="ml-2 whitespace-nowrap overflow-hidden"
            >
              {{ t(item.labelKey) }}
            </span>
          </button>
        </li>
      </ul>
    </nav>

    <!-- Bottom section -->
    <div class="border-t border-gray-200/60 dark:border-surface-border/40 py-2">
      <ul class="space-y-1" :class="isCollapsed ? 'px-1' : 'px-2'">
        <li v-for="item in bottomItems" :key="item.path">
          <button
            @click="navigateTo(item.path)"
            :class="[
              'w-full flex items-center rounded-[4px] transition-colors duration-100',
              isCollapsed
                ? 'justify-center p-2'
                : 'px-3 py-2 text-[13px]',
              route.path === item.path
                ? 'bg-primary-50/60 dark:bg-primary-500/10 text-primary-600 dark:text-brand-primary'
                : 'text-gray-600 dark:text-content-secondary hover:bg-black/[0.04] dark:hover:bg-white/[0.06]'
            ]"
            :title="isCollapsed ? t(item.labelKey) : undefined"
          >
            <svg v-if="item.icon === 'settings'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <svg v-else-if="item.icon === 'about'" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span
              v-if="!isCollapsed"
              class="ml-2 whitespace-nowrap overflow-hidden"
            >
              {{ t(item.labelKey) }}
            </span>
          </button>
        </li>
      </ul>

      <!-- Dark mode toggle + collapse toggle -->
      <div :class="isCollapsed ? 'px-1' : 'px-2'" class="pt-2">
        <div class="space-y-1">
          <button
            @click="toggleDarkMode"
            :class="[
              'w-full flex items-center rounded-[4px] transition-colors duration-100 text-gray-500 dark:text-content-muted hover:bg-black/[0.04] dark:hover:bg-white/[0.06]',
              isCollapsed ? 'justify-center p-2' : 'px-3 py-2 text-[13px]'
            ]"
            :title="isCollapsed ? (isDarkMode ? t('settings.themeLight') : t('settings.themeDark')) : undefined"
          >
            <svg v-if="isDarkMode" class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
            <svg v-else class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
            </svg>
            <span v-if="!isCollapsed" class="ml-2 whitespace-nowrap overflow-hidden">{{ isDarkMode ? t('settings.themeLight') : t('settings.themeDark') }}</span>
          </button>
          <button
            @click="toggleSidebar"
            :class="[
              'w-full flex items-center rounded-[4px] transition-colors duration-100 text-gray-500 dark:text-content-muted hover:bg-black/[0.04] dark:hover:bg-white/[0.06]',
              isCollapsed ? 'justify-center p-2' : 'px-3 py-2 text-[13px]'
            ]"
            :title="isCollapsed ? (isCollapsed ? t('sidebar.expand') : t('sidebar.collapse')) : undefined"
          >
            <svg
              class="w-5 h-5 shrink-0 transition-transform duration-200"
              :class="isCollapsed ? 'rotate-180' : ''"
              fill="none" stroke="currentColor" viewBox="0 0 24 24"
            >
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
            </svg>
            <span v-if="!isCollapsed" class="ml-2 whitespace-nowrap overflow-hidden">{{ t('sidebar.collapse') }}</span>
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>
