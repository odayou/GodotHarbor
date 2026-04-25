<script setup lang="ts">
import { ref } from 'vue'

const appVersion = '0.2.0'

const rustDeps = [
  { name: 'Tauri', version: '2.x', license: 'Apache-2.0 / MIT', url: 'https://tauri.app' },
  { name: 'Serde', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/serde-rs/serde' },
  { name: 'Tokio', version: '1.x', license: 'MIT', url: 'https://tokio.rs' },
  { name: 'UUID', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/uuid-rs/uuid' },
  { name: 'Chrono', version: '0.4', license: 'Apache-2.0 / MIT', url: 'https://github.com/chronotope/chrono' },
  { name: 'WalkDir', version: '2.x', license: 'Unlicense / MIT', url: 'https://github.com/BurntSushi/walkdir' },
  { name: 'Git2', version: '0.18', license: 'Apache-2.0 / MIT', url: 'https://github.com/rust-lang/git2-rs' },
  { name: 'Anyhow', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/dtolnay/anyhow' },
  { name: 'Reqwest', version: '0.12', license: 'Apache-2.0 / MIT', url: 'https://github.com/seanmonstar/reqwest' },
  { name: 'Zip', version: '2.x', license: 'MIT', url: 'https://github.com/zip-rs/zip2' },
]

const jsDeps = [
  { name: 'Vue.js', version: '3.x', license: 'MIT', url: 'https://vuejs.org' },
  { name: 'Vue Router', version: '4.x', license: 'MIT', url: 'https://router.vuejs.org' },
  { name: 'Pinia', version: '2.x', license: 'MIT', url: 'https://pinia.vuejs.org' },
  { name: 'Vite', version: '5.x', license: 'MIT', url: 'https://vitejs.dev' },
  { name: 'TypeScript', version: '5.x', license: 'Apache-2.0', url: 'https://www.typescriptlang.org' },
  { name: 'Tailwind CSS', version: '3.x', license: 'MIT', url: 'https://tailwindcss.com' },
]

const activeTab = ref<'about' | 'credits' | 'sponsor'>('about')
</script>

<template>
  <div class="space-y-6">
    <div class="flex gap-2 border-b border-gray-200 dark:border-gray-700 pb-2">
      <button
        @click="activeTab = 'about'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'about' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        关于
      </button>
      <button
        @click="activeTab = 'credits'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'credits' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        鸣谢
      </button>
      <button
        @click="activeTab = 'sponsor'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'sponsor' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        赞助
      </button>
    </div>

    <div v-if="activeTab === 'about'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
        <div class="w-20 h-20 mx-auto mb-4 bg-primary-100 dark:bg-primary-900/30 rounded-2xl flex items-center justify-center">
          <svg class="w-12 h-12 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">Godot Harbor</h1>
        <p class="text-gray-500 dark:text-gray-400 mt-2">v{{ appVersion }}</p>
        <p class="text-gray-600 dark:text-gray-400 mt-4 max-w-xl mx-auto">
          Godot Harbor 是一款独立的桌面应用，用于管理 Godot 引擎的插件、项目和引擎版本。
          让插件只需导入一次，即可被多个项目复用，通过符号链接实现零复制安装。
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">符号链接引擎</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">零复制安装，磁盘占用最小，一改全改</p>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">版本精细管理</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">同一插件多版本共存，项目按需绑定</p>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">团队协作</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">团队配置导入导出，环境一致性保证</p>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">技术栈</h3>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">🦀</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">Rust</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">后端核心</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">⚡</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">Tauri 2</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">桌面框架</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">💚</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">Vue 3</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">前端框架</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">🎨</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">Tailwind</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">CSS 框架</p>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Godot Harbor 是开源软件，基于 Apache-2.0 / MIT 许可证发布
        </p>
        <div class="flex justify-center gap-4 mt-4">
          <a href="https://github.com" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            GitHub 仓库
          </a>
          <a href="https://github.com" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            问题反馈
          </a>
          <a href="https://github.com" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            参与贡献
          </a>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'credits'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">鸣谢</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Godot Harbor 的诞生离不开以下开源项目和社区的支持。
        </p>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">Rust 依赖</h3>
        <div class="space-y-2 mb-6">
          <div
            v-for="dep in rustDeps"
            :key="dep.name"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
          >
            <div class="flex items-center gap-3">
              <a :href="dep.url" target="_blank" class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:underline">
                {{ dep.name }}
              </a>
              <span class="text-xs text-gray-500 dark:text-gray-400">v{{ dep.version }}</span>
            </div>
            <span class="text-xs text-gray-400 dark:text-gray-500">{{ dep.license }}</span>
          </div>
        </div>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">前端依赖</h3>
        <div class="space-y-2 mb-6">
          <div
            v-for="dep in jsDeps"
            :key="dep.name"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
          >
            <div class="flex items-center gap-3">
              <a :href="dep.url" target="_blank" class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:underline">
                {{ dep.name }}
              </a>
              <span class="text-xs text-gray-500 dark:text-gray-400">v{{ dep.version }}</span>
            </div>
            <span class="text-xs text-gray-400 dark:text-gray-500">{{ dep.license }}</span>
          </div>
        </div>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">灵感来源</h3>
        <div class="space-y-2">
          <div class="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">Godot Engine</p>
            <p class="text-xs text-gray-500 dark:text-gray-400">优秀的开源游戏引擎，Godot Harbor 为其生态而生</p>
          </div>
          <div class="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">GodotEnv</p>
            <p class="text-xs text-gray-500 dark:text-gray-400">声明式插件管理的先驱，harbor.json 的灵感来源</p>
          </div>
          <div class="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">gd-plug</p>
            <p class="text-xs text-gray-500 dark:text-gray-400">编辑器内插件管理的标杆，Git 版本冻结的思路参考</p>
          </div>
          <div class="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">godam</p>
            <p class="text-xs text-gray-500 dark:text-gray-400">Rust 编写的轻量 CLI，Asset Library API 集成的参考</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'sponsor'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
        <div class="w-16 h-16 mx-auto mb-4 bg-pink-100 dark:bg-pink-900/30 rounded-full flex items-center justify-center">
          <svg class="w-8 h-8 text-pink-600 dark:text-pink-400" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">支持 Godot Harbor</h2>
        <p class="text-gray-600 dark:text-gray-400 max-w-md mx-auto mb-6">
          Godot Harbor 是免费开源软件。如果您觉得它对您有帮助，可以考虑通过以下方式支持我们的开发。
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-yellow-100 dark:bg-yellow-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">☕</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Buy Me a Coffee</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            一次性的小额赞助，帮助我们保持开发热情。
          </p>
          <a href="https://buymeacoffee.com" target="_blank" class="inline-block px-4 py-2 bg-yellow-500 text-white rounded-lg hover:bg-yellow-600 text-sm">
            ☕ 请我喝咖啡
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center">
              <svg class="w-5 h-5 text-white dark:text-gray-900" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">GitHub Sponsors</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            通过 GitHub 赞助，按月或一次性支持开发者。
          </p>
          <a href="https://github.com/sponsors" target="_blank" class="inline-block px-4 py-2 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 rounded-lg hover:bg-gray-800 dark:hover:bg-gray-200 text-sm">
            ❤️ 成为赞助者
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">🌟</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Star on GitHub</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            给项目点一个 Star 是最简单的支持方式，也能帮助更多人发现 Godot Harbor。
          </p>
          <a href="https://github.com" target="_blank" class="inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm">
            ⭐ 给个 Star
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">🤝</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">参与贡献</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            代码贡献、Bug 反馈、功能建议、文档完善，每一种贡献都同样宝贵。
          </p>
          <a href="https://github.com" target="_blank" class="inline-block px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm">
            📖 贡献指南
          </a>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          感谢每一位支持者！您的支持是 Godot Harbor 持续发展的动力。
        </p>
      </div>
    </div>
  </div>
</template>
