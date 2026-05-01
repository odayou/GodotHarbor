<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue'
import { useCommandPalette } from '@/composables/useCommandPalette'
import type { SearchItem } from '@/composables/useCommandPalette'

const {
  isOpen,
  query,
  selectedIndex,
  groupedResults,
  filteredItems,
  closePalette,
  selectItem,
  moveSelection,
  selectCurrentItem,
  t
} = useCommandPalette()

const searchInput = ref<HTMLInputElement | null>(null)

watch(isOpen, async (open) => {
  if (open) {
    await nextTick()
    searchInput.value?.focus()
  }
})

const categoryLabels = computed(() => ({
  command: t('commandPalette.category.commands'),
  project: t('commandPalette.category.projects'),
  plugin: t('commandPalette.category.plugins'),
  engine: t('commandPalette.category.engines'),
  setting: t('commandPalette.category.settings'),
}))

function onOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    closePalette()
  }
}

function onKeyDown(e: KeyboardEvent) {
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      moveSelection(1)
      break
    case 'ArrowUp':
      e.preventDefault()
      moveSelection(-1)
      break
    case 'Enter':
      e.preventDefault()
      selectCurrentItem()
      break
    case 'Escape':
      e.preventDefault()
      closePalette()
      break
  }
}

function getFlatIndex(item: SearchItem): number {
  return filteredItems.value.indexOf(item)
}

function getHighlightSegments(text: string, searchQuery: string): Array<{ text: string; highlight: boolean }> {
  if (!searchQuery.trim()) return [{ text, highlight: false }]
  const lowerText = text.toLowerCase()
  const lowerQuery = searchQuery.toLowerCase()
  const segments: Array<{ text: string; highlight: boolean }> = []
  let lastIndex = 0
  let searchFrom = 0
  while (searchFrom < lowerText.length) {
    const idx = lowerText.indexOf(lowerQuery, searchFrom)
    if (idx === -1) break
    if (idx > lastIndex) {
      segments.push({ text: text.slice(lastIndex, idx), highlight: false })
    }
    segments.push({ text: text.slice(idx, idx + searchQuery.length), highlight: true })
    lastIndex = idx + searchQuery.length
    searchFrom = lastIndex
  }
  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), highlight: false })
  }
  return segments.length > 0 ? segments : [{ text, highlight: false }]
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/50 flex items-start justify-center pt-[15vh] z-[100]"
      @click="onOverlayClick"
    >
      <div
        class="w-full max-w-lg bg-white dark:bg-gray-800 rounded-xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden"
        @keydown="onKeyDown"
      >
        <div class="flex items-center px-4 border-b border-gray-200 dark:border-gray-700">
          <svg class="w-5 h-5 text-gray-400 dark:text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref="searchInput"
            v-model="query"
            type="text"
            :placeholder="t('commandPalette.placeholder')"
            class="w-full px-3 py-3 bg-transparent text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none text-sm"
          />
          <kbd class="hidden sm:inline-flex items-center px-2 py-0.5 text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 shrink-0">
            Esc
          </kbd>
        </div>

        <div class="max-h-80 overflow-y-auto">
          <template v-if="groupedResults.length > 0">
            <template v-for="group in groupedResults" :key="group.category">
              <div class="px-4 py-1.5 text-xs font-medium text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/30 sticky top-0">
                {{ categoryLabels[group.category] || group.category }}
              </div>
              <button
                v-for="item in group.items"
                :key="item.id"
                @click="selectItem(item)"
                @mouseenter="selectedIndex = getFlatIndex(item)"
                :class="[
                  'w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors',
                  getFlatIndex(item) === selectedIndex
                    ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300'
                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50'
                ]"
              >
                <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                  :class="getFlatIndex(item) === selectedIndex
                    ? 'bg-primary-100 dark:bg-primary-800/30'
                    : 'bg-gray-100 dark:bg-gray-700'"
                >
                  <svg v-if="item.icon === 'home'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                  </svg>
                  <svg v-else-if="item.icon === 'folder'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  <svg v-else-if="item.icon === 'puzzle'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                  </svg>
                  <svg v-else-if="item.icon === 'link'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                  <svg v-else-if="item.icon === 'engine'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  <svg v-else-if="item.icon === 'settings'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  <svg v-else-if="item.icon === 'theme'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                  </svg>
                  <svg v-else-if="item.icon === 'sidebar'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                  </svg>
                  <svg v-else-if="item.icon === 'scan'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  <svg v-else-if="item.icon === 'import'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  <svg v-else-if="item.icon === 'language'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
                  </svg>
                  <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <span class="text-sm font-medium truncate block">
                    <template v-for="(segment, idx) in getHighlightSegments(item.label, query)" :key="idx">
                      <mark v-if="segment.highlight" class="bg-yellow-200 dark:bg-yellow-800 text-inherit rounded px-0.5">{{ segment.text }}</mark>
                      <span v-else>{{ segment.text }}</span>
                    </template>
                  </span>
                </div>
                <svg
                  v-if="getFlatIndex(item) === selectedIndex"
                  class="w-4 h-4 text-primary-500 dark:text-primary-400 shrink-0"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </button>
            </template>
          </template>

          <div v-else class="px-4 py-8 text-center">
            <svg class="w-10 h-10 mx-auto text-gray-300 dark:text-gray-600 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <p class="text-sm text-gray-500 dark:text-gray-400">{{ t('commandPalette.noResults') }}</p>
          </div>
        </div>

        <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 flex items-center gap-4 text-xs text-gray-400 dark:text-gray-500">
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 text-[10px]">↑↓</kbd>
            {{ t('commandPalette.navigate') }}
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 text-[10px]">↵</kbd>
            {{ t('commandPalette.select') }}
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 text-[10px]">Esc</kbd>
            {{ t('commandPalette.close') }}
          </span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
mark {
  background-color: transparent;
  color: inherit;
  font-weight: 700;
  text-decoration: underline;
  text-decoration-color: theme('colors.primary.400');
  text-underline-offset: 2px;
}
</style>
