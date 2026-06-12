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
  selectByShortcutKey,
  t,
  showEngineSelectDialog,
  engineSelectProject,
  matchedEngines,
  isLoadingEngines,
  launchWithEngine,
  closeEngineSelectDialog,
  getMatchLevelClass,
  getMatchLevelLabel,
  getMatchLevelDesc,
} = useCommandPalette()

const searchInput = ref<HTMLInputElement | null>(null)
const scrollContainer = ref<HTMLElement | null>(null)

watch(isOpen, async (open) => {
  if (open) {
    await nextTick()
    searchInput.value?.focus()
  }
})

watch(selectedIndex, async () => {
  await nextTick()
  const selectedEl = document.querySelector('[data-palette-index].palette-selected')
  if (selectedEl) {
    selectedEl.scrollIntoView({ block: 'nearest' })
  }
})

const categoryLabels = computed(() => ({
  navigation: t('commandPalette.category.navigation'),
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
  if (e.key >= '1' && e.key <= '9' && !e.ctrlKey && !e.altKey && !e.metaKey && !query.value.trim()) {
    e.preventDefault()
    selectByShortcutKey(e.key)
    return
  }

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
        class="w-full max-w-lg bg-white dark:bg-surface-card rounded-lg shadow-2xl border border-gray-200 dark:border-surface-border overflow-hidden"
        @keydown="onKeyDown"
      >
        <div class="flex items-center px-4 border-b border-gray-200 dark:border-surface-border">
          <svg class="w-5 h-5 text-gray-400 dark:text-content-muted shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref="searchInput"
            v-model="query"
            type="text"
            :placeholder="t('commandPalette.placeholder')"
            class="w-full px-3 py-3 bg-transparent text-gray-900 dark:text-content-primary placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none text-sm"
          />
          <kbd class="hidden sm:inline-flex items-center px-2 py-0.5 text-xs text-gray-400 dark:text-content-muted bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border shrink-0">
            Esc
          </kbd>
        </div>

        <div ref="scrollContainer" class="max-h-80 overflow-y-auto">
          <template v-if="groupedResults.length > 0">
            <template v-for="group in groupedResults" :key="group.category">
              <div class="px-4 py-1.5 text-xs font-medium text-gray-500 dark:text-content-muted bg-gray-50 dark:bg-surface-layer/50 sticky top-0">
                {{ categoryLabels[group.category] || group.category }}
              </div>
              <button
                v-for="item in group.items"
                :key="item.id"
                :data-palette-index="getFlatIndex(item)"
                :class="[
                  'w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors',
                  getFlatIndex(item) === selectedIndex ? 'bg-primary-50 dark:bg-surface-hover text-primary-700 dark:text-content-secondary palette-selected' : 'text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover/50'
                ]"
                @click="selectItem(item)"
                @mouseenter="selectedIndex = getFlatIndex(item)"
              >
                <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                  :class="getFlatIndex(item) === selectedIndex
                    ? 'bg-surface-hover dark:bg-surface-hover'
                    : 'bg-gray-100 dark:bg-surface-hover'"
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
                  <svg v-else-if="item.icon === 'updates'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  <svg v-else-if="item.icon === 'about'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
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
                <kbd v-if="item.shortcutKey && !query.trim()"
                  class="px-1.5 py-0.5 text-[11px] font-mono text-gray-400 dark:text-content-muted bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border shrink-0 min-w-[20px] text-center"
                >{{ item.shortcutKey }}</kbd>
                <svg
                  v-if="getFlatIndex(item) === selectedIndex && !item.shortcutKey"
                  class="w-4 h-4 text-primary-500 dark:text-brand-primary shrink-0"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </button>
            </template>
          </template>

          <div v-else class="px-4 py-8 text-center">
            <svg class="w-10 h-10 mx-auto text-gray-300 dark:text-content-muted mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('commandPalette.noResults') }}</p>
          </div>
        </div>

        <div class="px-4 py-2 border-t border-gray-200 dark:border-surface-border flex items-center gap-4 text-xs text-gray-400 dark:text-content-muted">
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border text-[10px]">↑↓</kbd>
            {{ t('commandPalette.navigate') }}
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border text-[10px]">↵</kbd>
            {{ t('commandPalette.select') }}
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border text-[10px]">1-9</kbd>
            {{ t('commandPalette.quickSelect') }}
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-surface-hover rounded border border-gray-200 dark:border-surface-border text-[10px]">Esc</kbd>
            {{ t('commandPalette.close') }}
          </span>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showEngineSelectDialog && engineSelectProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[110]" @click="closeEngineSelectDialog">
      <div class="dialog-container w-full max-w-md max-h-[80vh] flex flex-col" @click.stop>
        <h3 class="dialog-title">{{ t('projects.openWithEngine') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
          {{ t('projects.openWithEngineDesc') }}
          <span class="font-mono text-xs bg-gray-100 dark:bg-surface-hover px-1.5 py-0.5 rounded ml-1">Godot {{ engineSelectProject.godot_version }}</span>
        </p>

        <div v-if="isLoadingEngines" class="flex-1 flex items-center justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
        </div>

        <div v-else-if="matchedEngines.length === 0" class="flex-1 py-8 text-center">
          <svg class="mx-auto h-10 w-10 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.noMatchingEngines') }}</p>
          <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('projects.noMatchingEnginesDesc') }}</p>
          <button
            @click="$router.push('/engines'); closePalette()"
            class="mt-3 px-3 py-1.5 text-xs font-medium bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
          >
            {{ t('projects.goToEngines') }}
          </button>
        </div>

        <div v-else class="flex-1 overflow-y-auto space-y-2 min-h-0">
          <button
            v-for="me in matchedEngines"
            :key="me.engine.engine_id"
            @click="launchWithEngine(me.engine.engine_id)"
            :class="[
              'w-full text-left p-3 rounded-lg border transition-colors',
              me.engine.engine_id === engineSelectProject?.last_used_engine_id
                ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover'
                : 'border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-surface-border hover:bg-primary-50 dark:hover:bg-surface-hover'
            ]"
          >
            <div class="flex items-center justify-between">
              <div class="min-w-0 flex-1">
                <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                  {{ me.engine.name }}
                  <span v-if="me.engine.engine_id === engineSelectProject?.last_used_engine_id" class="text-xs text-primary-600 dark:text-brand-primary font-normal">{{ t('projects.lastUsedEngine') }}</span>
                </div>
                <div class="text-xs text-gray-500 dark:text-content-muted mt-0.5 font-mono flex items-center gap-1.5">v{{ me.engine.version }}<span v-if="me.engine.is_mono" class="text-[10px] px-1 py-0.5 rounded bg-purple-100 dark:bg-surface-hover text-purple-700 dark:text-content-secondary font-sans font-medium">{{ t('projects.monoLabel') }}</span></div>
              </div>
              <span
                :class="['text-xs px-2 py-0.5 rounded-full font-medium ml-2 flex-shrink-0', getMatchLevelClass(me.match_level)]"
                :title="getMatchLevelDesc(me.match_level)"
              >
                {{ getMatchLevelLabel(me.match_level) }}
              </span>
            </div>
            <div v-if="me.match_level !== 'exact'" class="mt-1.5 text-xs text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
              <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" /></svg>
              {{ getMatchLevelDesc(me.match_level) }}
            </div>
          </button>
        </div>

        <div class="flex justify-end mt-3 pt-2 border-t border-gray-200 dark:border-surface-border">
          <button
            @click="closeEngineSelectDialog"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
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
