<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from '@/composables/useI18n'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()

const menuItems = [
  { path: '/', icon: 'home', labelKey: 'nav.home' },
  { path: '/projects', icon: 'folder', labelKey: 'nav.projects' },
  { path: '/plugins', icon: 'puzzle', labelKey: 'nav.plugins' },
  { path: '/linker', icon: 'link', labelKey: 'nav.linker' },
  { path: '/engines', icon: 'engine', labelKey: 'nav.engines' },
  { path: '/roadmap', icon: 'roadmap', labelKey: 'nav.roadmap' },
  { path: '/about', icon: 'about', labelKey: 'nav.about' },
  { path: '/settings', icon: 'settings', labelKey: 'nav.settings' }
]

const navigateTo = (path: string) => {
  router.push(path)
}
</script>

<template>
  <aside class="w-64 bg-white dark:bg-gray-800 shadow-lg flex flex-col">
    <div class="h-16 flex items-center justify-center border-b border-gray-200 dark:border-gray-700 gap-2">
      <svg class="w-8 h-8" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="sbBg" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#4A90D9"/>
            <stop offset="100%" stop-color="#2B6CB0"/>
          </linearGradient>
          <linearGradient id="sbGear" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#5DA0E0"/>
            <stop offset="100%" stop-color="#3B7CC5"/>
          </linearGradient>
          <linearGradient id="sbInner" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#3A7BBF"/>
            <stop offset="100%" stop-color="#2B5F9E"/>
          </linearGradient>
        </defs>
        <rect width="512" height="512" rx="96" fill="url(#sbBg)"/>
        <g transform="translate(256,220)">
          <path d="M 0,-170 L 28,-158 L 32,-122 L 64,-126 L 80,-98 L 56,-74 L 76,-48 L 100,-56 L 120,-30 L 98,-8 L 118,20 L 142,16 L 150,46 L 126,56 L 134,86 L 108,90 L 104,120 L 78,116 L 66,144 L 40,132 L 20,156 L 0,140 L -20,156 L -40,132 L -66,144 L -78,116 L -104,120 L -108,90 L -134,86 L -126,56 L -150,46 L -142,16 L -118,20 L -98,-8 L -120,-30 L -100,-56 L -76,-48 L -56,-74 L -80,-98 L -64,-126 L -32,-122 L -28,-158 Z" fill="url(#sbGear)"/>
          <circle cx="0" cy="0" r="72" fill="url(#sbInner)"/>
          <g fill="none" stroke="#FFFFFF" stroke-width="14" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="0" cy="-30" r="18" stroke-width="12"/>
            <line x1="0" y1="-12" x2="0" y2="80"/>
            <line x1="-36" y1="30" x2="36" y2="30"/>
            <path d="M -36 30 Q -48 60 -40 90 Q -32 70 -20 60"/>
            <path d="M 36 30 Q 48 60 40 90 Q 32 70 20 60"/>
          </g>
          <path d="M -8 80 Q 0 96 8 80" fill="#FFFFFF" stroke="none"/>
        </g>
        <path d="M 140 400 Q 170 386 200 400 Q 230 414 260 400 Q 290 386 320 400 Q 350 414 380 400" fill="none" stroke="#FFFFFF" stroke-width="5" opacity="0.35"/>
        <path d="M 155 420 Q 185 406 215 420 Q 245 434 275 420 Q 305 406 335 420 Q 365 434 395 420" fill="none" stroke="#FFFFFF" stroke-width="3.5" opacity="0.2"/>
      </svg>
      <h1 class="text-lg font-bold text-primary-600 dark:text-primary-400">
        Godot Harbor
      </h1>
    </div>
    
    <nav class="flex-1 py-4">
      <ul class="space-y-1 px-3">
        <li v-for="item in menuItems" :key="item.path">
          <button
            @click="navigateTo(item.path)"
            :class="[
              'w-full flex items-center px-4 py-3 text-sm font-medium rounded-lg transition-colors',
              route.path === item.path
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            ]"
          >
            <span class="mr-3">
              <svg v-if="item.icon === 'home'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
              <svg v-else-if="item.icon === 'folder'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <svg v-else-if="item.icon === 'puzzle'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
              </svg>
              <svg v-else-if="item.icon === 'link'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              <svg v-else-if="item.icon === 'engine'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              <svg v-else-if="item.icon === 'roadmap'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
              </svg>
              <svg v-else-if="item.icon === 'about'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <svg v-else-if="item.icon === 'settings'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </span>
            {{ t(item.labelKey) }}
          </button>
        </li>
      </ul>
    </nav>
  </aside>
</template>
