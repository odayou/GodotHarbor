================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 1 |
================================================================================
                  { key: 'pluginsDir', path: storagePaths.plugins_dir },
                  { key: 'enginesDir', path: storagePaths.engines_dir },
                  { key: 'cacheDir', path: storagePaths.cache_dir },
                  { key: 'logsDir', path: storagePaths.logs_dir },
                  { key: 'hotUpdatesDir', path: storagePaths.hot_updates_dir },
                  { key: 'settingsFile', path: storagePaths.settings_file },
                  { key: 'projectsFile', path: storagePaths.projects_file },
                  { key: 'enginesFile', path: storagePaths.engines_file }
                ]" :key="item.key">
                  <td class="px-3 py-1.5 text-gray-700 dark:text-content-secondary whitespace-nowrap">{{ t(`settings.storage.${item.key}`) }}</td>
                  <td class="px-3 py-1.5 font-mono text-gray-600 dark:text-content-muted break-all">{{ item.path }}</td>
                  <td class="px-3 py-1.5 text-right"><button @click="openPath(item.path)" class="text-primary-600 dark:text-primary-400 hover:underline">{{ t('settings.storage.open') }}</button></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.dataOps') }}</h2>
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.buttons.backup') }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('settings.backup.desc') }}</p>
              </div>
              <div class="flex gap-2">
                <button @click="showBackupDialog = true" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors text-sm">{{ t('settings.buttons.backup') }}</button>
                <button @click="showRestoreDialog = true" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors text-sm">{{ t('settings.backup.restore') }}</button>
              </div>
            </div>
            <div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-surface-border">
              <div>
                <p class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.resetDataLabel') }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('settings.resetDataDesc') }}</p>
              </div>
              <button @click="confirmResetData" class="px-4 py-2 border border-red-300 dark:border-red-600 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-800/20 transition-colors text-sm">{{ t('settings.resetDataLabel') }}</button>
            </div>
          </div>
        </div>
      </div>
      <div v-show="activeSection === 'updates'" class="space-y-6">
        <div class="card p-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.updates.autoCheck') }}</h2>
            <button @click="router.push('/updates')" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium transition-colors">
              {{ t('settings.updates.checkNow') }}
            </button>
          </div>
          <div class="space-y-3">
            <label class="flex items-center gap-3 cursor-pointer">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 2 |
================================================================================
              <input type="checkbox" v-model="settings.auto_check_app_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckAppUpdates') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_check_plugin_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckPluginUpdates') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_check_engine_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckEngineUpdates') }}</span>
            </label>
            <div v-if="settings.auto_check_engine_updates" class="pl-7 pt-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.pluginRepo.updateChannels') }}</label>
              <div class="flex flex-wrap gap-3">
                <label v-for="ch in updateChannelOptions" :key="ch.value" class="flex items-center gap-2 cursor-pointer">
                  <input type="checkbox" :value="ch.value" v-model="settings.engine_update_channels" class="w-4 h-4 text-primary-600 rounded" />
                  <span class="text-sm text-gray-700 dark:text-content-secondary">{{ ch.label }}</span>
                </label>
              </div>
              <p class="text-xs text-gray-400 dark:text-content-muted mt-1.5">{{ t('settings.pluginRepo.updateChannelsHint') }}</p>
            </div>
            <div class="pt-2">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.pluginRepo.checkInterval') }}</label>
              <input type="number" v-model.number="settings.update_check_interval_hours" min="1" max="168"
                class="w-32 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm" />
            </div>
          </div>
        </div>
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.engineMirror.title') }}</h2>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('settings.engineMirror.desc') }}</p>

          <div class="mb-5 p-4 rounded-lg border border-gray-200 dark:border-surface-border">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.networkProxy.githubApiProxy') }}</label>
            <input
              v-model="settings.github_api_proxy"
              type="text"
              :placeholder="t('settings.networkProxy.githubApiProxyPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.networkProxy.githubApiProxyHint') }}</p>
          </div>

          <div class="mb-5 p-4 rounded-lg border border-gray-200 dark:border-surface-border">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.networkProxy.assetLibraryMirror') }}</label>
            <input
              v-model="settings.asset_library_mirror"
              type="text"
              :placeholder="t('settings.networkProxy.assetLibraryMirrorPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 3 |
================================================================================
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.networkProxy.assetLibraryMirrorHint') }}</p>
          </div>

          <div class="mb-5 p-4 rounded-lg border border-gray-200 dark:border-surface-border">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.networkProxy.assetApiMode') }}</label>
            <div class="flex gap-3">
              <label v-for="mode in (['auto', 'legacy', 'new_store'] as const)" :key="mode" class="flex items-center gap-2 cursor-pointer">
                <input type="radio" v-model="settings.asset_api_mode" :value="mode" class="accent-blue-600" />
                <span class="text-sm text-gray-700 dark:text-content-primary">{{ t(`settings.networkProxy.assetApiMode_${mode}`) }}</span>
              </label>
            </div>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.networkProxy.assetApiModeHint') }}</p>
          </div>

          <div class="space-y-3">
            <div v-for="mirror in (settings.engine_mirrors || [])" :key="mirror.id"
              class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 dark:border-surface-border"
              :class="{ 'opacity-60': !mirror.enabled }"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ mirror.name }}</span>
                  <span v-if="mirror.is_official" class="px-1.5 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">{{ t('settings.engineMirror.official') }}</span>
                  <span v-else class="px-1.5 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-surface-hover dark:text-content-muted">{{ t('settings.engineMirror.custom') }}</span>
                </div>
                <span class="text-xs text-gray-500 dark:text-content-muted truncate block mt-0.5">{{ mirror.base_url }}</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  @click="toggleMirrorEnabled(mirror.id)"
                  :class="['px-2 py-1 rounded text-xs font-medium transition-colors', mirror.enabled ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-500 dark:bg-surface-hover dark:text-content-muted']"
                >
                  {{ mirror.enabled ? t('settings.engineMirror.enabled') : t('settings.engineMirror.disabled') }}
                </button>
                <button
                  @click="openEditMirror(mirror)"
                  class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-1 rounded hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
                  :title="t('settings.engineMirror.edit')"
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                </button>
                <button
                  v-if="!mirror.is_official"
                  @click="removeMirror(mirror.id)"
                  class="text-red-500 hover:text-red-700 p-1 rounded hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                  :title="t('settings.engineMirror.remove')"
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                </button>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 4 |
================================================================================
              </div>
            </div>
            <button
              @click="openAddMirror"
              class="px-4 py-2 border border-dashed border-gray-300 dark:border-surface-border text-gray-600 dark:text-content-muted rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm w-full"
            >
              + {{ t('settings.engineMirror.addMirror') }}
            </button>
          </div>
        </div>
      </div>
      </div>
    </div>
    </div>

    <Transition
      enter-active-class="transition-all duration-300"
      enter-from-class="translate-y-full opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition-all duration-200"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-full opacity-0"
    >
      <div v-if="isDirty" class="fixed bottom-0 left-0 right-0 bg-white dark:bg-surface-card border-t border-primary-200 dark:border-primary-800 shadow-lg z-40 px-6 py-3 flex items-center justify-between">
        <p class="text-sm text-gray-600 dark:text-content-muted">{{ t('settings.unsavedChanges') }}</p>
        <div class="flex gap-3">
          <button @click="loadSettings" class="px-4 py-2 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm">{{ t('settings.discardChanges') }}</button>
          <button @click="saveSettingsWithMigrationCheck" :disabled="isLoading" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm">{{ t('settings.save') }}</button>
        </div>
      </div>
    </Transition>
  </div>

  <Teleport to="body">
  <div v-if="showLogs" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLogs = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-3xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.logs.title') }}</h3>
          <div class="flex items-center gap-3">
            <button
              @click="logSortOrder = logSortOrder === 'newest' ? 'oldest' : 'newest'"
              class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 flex items-center gap-1"
            >
              <svg class="w-3.5 h-3.5" :class="{ 'rotate-180': logSortOrder === 'oldest' }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
              {{ logSortOrder === 'newest' ? t('settings.logs.sortNewest') : t('settings.logs.sortOldest') }}
            </button>
            <button @click="showLogs = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
              <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            </button>
          </div>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 5 |
================================================================================
        </div>
        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-if="sortedLogs.length === 0" class="text-center py-8 text-gray-500 dark:text-content-muted">{{ t('settings.logs.empty') }}</div>
          <div v-for="(log, index) in sortedLogs" :key="index" :class="['p-3 rounded-lg border', log.level === 'error' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-surface-hover border-gray-200 dark:border-surface-border']">
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-2">
                <span :class="['px-2 py-0.5 rounded text-xs font-medium', log.level === 'error' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300' : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300']">{{ log.level === 'error' ? t('settings.logs.error') : t('settings.logs.success') }}</span>
                <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ log.action }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500 dark:text-content-muted">{{ formatTime(log.timestamp) }}</span>
                <button v-if="log.level === 'error'" @click="copyError(log)" class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400">{{ t('settings.logs.copy') }}</button>
              </div>
            </div>
            <p v-if="log.target" class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.logs.target', { target: log.target }) }}</p>
            <p :class="['text-sm mt-1', log.level === 'error' ? 'text-red-700 dark:text-red-300' : 'text-gray-600 dark:text-content-muted']">{{ log.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showBackupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBackupDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.backup.title') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">
          {{ t('settings.backup.desc') }}
        </p>
        <div class="flex gap-2 mb-4">
          <input
            v-model="backupPath"
            type="text"
            readonly
            :placeholder="t('settings.backup.selectDir')"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
          />
          <button
            @click="selectBackupPath"
            class="btn-secondary text-sm whitespace-nowrap"
          >
            {{ t('settings.backup.browse') }}
          </button>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-800 dark:text-yellow-200">
            <strong>{{ t('settings.backup.warning') }}</strong>
          </p>
        </div>
        <div class="flex justify-end space-x-3">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 6 |
================================================================================
          <button
            @click="showBackupDialog = false; backupPath = ''"
            class="btn-secondary"
          >
            {{ t('settings.backup.cancel') }}
          </button>
          <button
            @click="performBackup"
            :disabled="isBackingUp || !backupPath"
            class="btn-primary disabled:opacity-50"
          >
            {{ isBackingUp ? t('settings.backup.backupping') : t('settings.backup.backup') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRestoreDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRestoreDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.backup.restoreTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">{{ t('settings.backup.restoreDesc') }}</p>
        <div class="flex gap-2 mb-4">
          <input v-model="backupPath" type="text" readonly :placeholder="t('settings.backup.selectDir')" class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm" />
          <button @click="selectBackupPath" class="btn-secondary text-sm whitespace-nowrap">{{ t('settings.backup.browse') }}</button>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-800 dark:text-yellow-200"><strong>{{ t('settings.backup.restoreWarning') }}</strong></p>
        </div>
        <div class="flex justify-end space-x-3">
          <button @click="showRestoreDialog = false; backupPath = ''" class="btn-secondary">{{ t('settings.backup.cancel') }}</button>
          <button @click="showRestoreConfirm = true" :disabled="isRestoring || !backupPath" class="btn-primary disabled:opacity-50">{{ isRestoring ? t('settings.backup.restoring') : t('settings.backup.restore') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <ConfirmDialog
      v-model="showRestoreConfirm"
      :title="t('settings.storage.backup.restoreConfirm')"
      :description="t('settings.storage.backup.restoreConfirmDesc')"
      :confirm-text="t('settings.storage.backup.restore')"
      confirm-color="red"
      @confirm="performRestore"
    />

  </Teleport>


================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 7 |
================================================================================
  <Teleport to="body">
    <div v-if="showDataMigrateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDataMigrateDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.storage.migrateTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-secondary mb-3">
          {{ t('settings.storage.migrateDescription') }}
        </p>
        <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 mb-3 text-xs font-mono space-y-1">
          <div class="text-red-500 dark:text-red-400">{{ t('settings.storage.migrateFrom') }}: {{ storagePaths?.app_data_dir }}</div>
          <div class="text-green-500 dark:text-green-400">{{ t('settings.storage.migrateTo') }}: {{ pendingDataDir }}</div>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-700 dark:text-yellow-400">{{ t('settings.storage.migrateWarning') }}</p>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showDataMigrateDialog = false; saveSettings()" :disabled="isMigratingData" class="btn-secondary">{{ t('settings.pluginRepo.skipMigration') }}</button>
          <button @click="executeDataMigration" :disabled="isMigratingData" class="btn-primary disabled:opacity-50">
            {{ isMigratingData ? t('settings.storage.migrating') : t('settings.storage.migrateButton') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showResetConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="showResetConfirm = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md max-h-[90vh] overflow-y-auto shadow-xl" @click.stop>
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.resetDataConfirmTitle') }}</h3>
        </div>

        <div class="flex items-center justify-center gap-2 mb-6">
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 1 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-surface-hover text-gray-500 dark:text-content-muted']">1</div>
          <div :class="['flex-1 h-1', resetStep >= 2 ? 'bg-primary-600' : 'bg-gray-200 dark:bg-surface-hover']"></div>
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 2 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-surface-hover text-gray-500 dark:text-content-muted']">2</div>
        </div>

        <div v-if="resetStep === 1" class="mb-6">
          <p class="text-sm text-gray-600 dark:text-content-secondary mb-4">
            {{ t('settings.resetDataStep1Desc') }}
          </p>
          <ul class="text-sm text-gray-500 dark:text-content-secondary space-y-2 mb-4 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            <li>{{ t('settings.resetDataItem.projects') }}</li>
            <li>{{ t('settings.resetDataItem.plugins') }}</li>
            <li>{{ t('settings.resetDataItem.engines') }}</li>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 8 |
================================================================================
            <li>{{ t('settings.resetDataItem.bindings') }}</li>
            <li>{{ t('settings.resetDataItem.settings') }}</li>
          </ul>
          <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 mb-4">
            <p class="text-sm text-blue-800 dark:text-blue-300">
              {{ t('settings.resetDataAutoBackup') }}
            </p>
          </div>
          <button @click="goToStep(2)" class="w-full btn-primary">
            {{ t('settings.resetDataStep1Continue') }}
          </button>
        </div>

        <div v-if="resetStep === 2" class="mb-6">
          <p class="text-sm text-gray-600 dark:text-content-secondary mb-4">
            {{ t('settings.resetDataStep2NewDesc') }}
          </p>
          <div class="flex gap-3">
            <input
              v-model="backupFingerprint"
              type="text"
              :placeholder="t('settings.resetDataStep2NewPlaceholder')"
              class="flex-1 px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary placeholder-gray-400"
            />
            <button @click="selectResetBackupPath" class="px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary transition-colors">
              {{ t('settings.buttons.select') }}
            </button>
          </div>
          <div class="flex justify-end gap-3 mt-4">
            <button @click="goToStep(1)" class="btn-secondary">{{ t('common.back') }}</button>
            <button @click="performReset" :disabled="isResetting || !backupFingerprint.trim()" class="btn-primary disabled:opacity-50">
              {{ isResetting ? t('settings.resetting') : t('settings.confirmReset') }}
            </button>
          </div>
        </div>

        <button @click="showResetConfirm = false" class="w-full mt-4 text-sm text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-gray-200">
          {{ t('common.cancel') }}
        </button>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showMirrorDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMirrorDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ editingMirror ? t('settings.engineMirror.editMirror') : t('settings.engineMirror.addMirror') }}</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorName') }}</label>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 9 |
================================================================================
            <input
              v-model="mirrorFormName"
              type="text"
              :placeholder="t('settings.engineMirror.mirrorNamePlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorUrl') }}</label>
            <input
              v-model="mirrorFormUrl"
              type="text"
              :placeholder="t('settings.engineMirror.mirrorUrlPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.engineMirror.urlHint') }}</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorType') }}</label>
            <select
              v-model="mirrorFormType"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            >
              <option value="github_api">GitHub API</option>
              <option value="direct">{{ t('settings.engineMirror.mirrorTypeDirect') }}</option>
            </select>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.engineMirror.mirrorTypeHint') }}</p>
          </div>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="mirrorFormEnabled" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.engineMirror.enableMirror') }}</span>
          </label>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showMirrorDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveMirror"
            :disabled="!mirrorFormName.trim() || !mirrorFormUrl.trim()"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 10 |
================================================================================
  </Teleport>

  <Teleport to="body">
    <div v-if="showUnsavedDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showUnsavedDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('settings.unsavedTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-6">{{ t('settings.unsavedDesc') }}</p>
        <div class="flex justify-end gap-3">
          <button @click="discardChanges" class="px-4 py-2 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm">{{ t('settings.discardChanges') }}</button>
          <button @click="saveAndLeave" class="btn-primary text-sm">{{ t('settings.saveAndLeave') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

// File: src\views\Templates.vue
<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { Template, TemplateCategory, TemplateInstantiationProgress } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { isOnline } from '@/composables/useNetworkStatus'
import EmptyState from '@/components/EmptyState.vue'
import SkeletonList from '@/components/SkeletonList.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t } = useI18n()
const router = useRouter()

const templates = ref<Template[]>([])
const isLoading = ref(true)
const loadError = ref<string | null>(null)
const categoryFilter = ref<TemplateCategory | 'all'>('all')
const searchQuery = ref('')

const showDetailDialog = ref(false)
const selectedTemplate = ref<Template | null>(null)

const showCreateDialog = ref(false)
const createProjectName = ref('')
const createTargetDir = ref('')
const enableMobileSupport = ref(false)
const isCreating = ref(false)

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 11 |
================================================================================
const createProgress = ref<TemplateInstantiationProgress | null>(null)
const projectNameError = ref('')

const isValidProjectName = computed(() => {
  const name = createProjectName.value.trim()
  if (!name) return false
  if (/[<>:"/\\|?*]/.test(name)) return false
  if (name.startsWith('.') || name.endsWith('.')) return false
  if (name.length > 200) return false
  return true
})

const validateProjectName = () => {
  const name = createProjectName.value.trim()
  if (!name) {
    projectNameError.value = ''
    return
  }
  if (/[<>:"/\\|?*]/.test(name)) {
    projectNameError.value = t('templates.invalidChars') || '项目名包含非法字符'
  } else if (name.startsWith('.') || name.endsWith('.')) {
    projectNameError.value = t('templates.invalidStartEnd') || '项目名不能以点号开头或结尾'
  } else {
    projectNameError.value = ''
  }
}

const showImportDialog = ref(false)
const importUrl = ref('')
const isImporting = ref(false)

const showGenerateFromProjectDialog = ref(false)
const generateProjectId = ref('')
const generateTemplateName = ref('')
const generateCategory = ref<TemplateCategory>('Custom')
const isGenerating = ref(false)
const projects = ref<any[]>([])

const showDeleteConfirm = ref(false)
const deleteTargetId = ref('')

useDialogEscape(showDetailDialog)
useDialogEscape(showCreateDialog)
useDialogEscape(showImportDialog)
useDialogEscape(showGenerateFromProjectDialog)

let unlistenProgress: UnlistenFn | null = null

onMounted(async () => {
  await loadTemplates()

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 12 |
================================================================================
  try {
    projects.value = await api.getProjects()
  } catch { /* ignore */ }
  unlistenProgress = await listen('template-instantiation-progress', (event) => {
    createProgress.value = event.payload as TemplateInstantiationProgress
  })
})

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
})

const loadTemplates = async () => {
  isLoading.value = true
  loadError.value = null
  try {
    await api.ensureBuiltinTemplates()
    templates.value = await api.listHubTemplates()
  } catch (e: any) {
    loadError.value = e?.toString() || 'Failed to load templates'
  } finally {
    isLoading.value = false
  }
}

const filteredTemplates = computed(() => {
  return templates.value.filter(tpl => {
    const matchesCategory = categoryFilter.value === 'all' || tpl.category === categoryFilter.value
    const matchesSearch = searchQuery.value === '' ||
      tpl.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.tags.some(tag => tag.toLowerCase().includes(searchQuery.value.toLowerCase()))
    return matchesCategory && matchesSearch
  })
})

const categories = computed(() => {
  const cats = new Set<TemplateCategory>(templates.value.map(t => t.category))
  return ['all', ...Array.from(cats)] as const
})

const categoryIcon = (cat: TemplateCategory | 'all') => {
  switch (cat) {
    case 'all': return '📋'
    case 'Starter2D': return '🎮'
    case 'Starter3D': return '🌐'
    case 'RPG': return '⚔️'
    case 'Platformer': return '🏃'

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 13 |
================================================================================
    case 'Multiplayer': return '👥'
    case 'Mobile': return '📱'
    case 'Blank': return '📄'
    case 'Custom': return '🔧'
    default: return '📁'
  }
}

const openDetail = (tpl: Template) => {
  selectedTemplate.value = tpl
  showDetailDialog.value = true
}

const openCreateDialog = async (tpl: Template) => {
  selectedTemplate.value = tpl
  createProjectName.value = ''
  createProgress.value = null
  projectNameError.value = ''
  enableMobileSupport.value = false
  if (!createTargetDir.value) {
    try {
      const paths = await api.getStoragePaths()
      const docsDir = paths.app_data_dir.replace(/[/\\]GodotHarbor[/\\]?$/, '')
      createTargetDir.value = docsDir
    } catch {
      createTargetDir.value = ''
    }
  }
  showCreateDialog.value = true
}

const selectTargetDir = async () => {
  const selected = await open({ directory: true, multiple: false, title: t('projects.selectDir') || 'Select Directory' })
  if (selected) {
    createTargetDir.value = selected as string
  }
}

const lastCreatedProjectId = ref('')

const handleCreate = async () => {
  if (!selectedTemplate.value || !createProjectName.value.trim() || !createTargetDir.value.trim()) return
  if (!isValidProjectName.value) return

  isCreating.value = true
  createProgress.value = null
  try {
    const result = await api.instantiateTemplate(
      selectedTemplate.value.template_id,
      createProjectName.value.trim(),

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 14 |
================================================================================
      createTargetDir.value.trim(),
      enableMobileSupport.value
    )
    showCreateDialog.value = false
    lastCreatedProjectId.value = result.project_id

    if (result.failed_plugins.length > 0) {
      const details = result.failed_plugins.join('\n')
      toast.warning(`${t('templates.createSuccess')} (${result.failed_plugins.length} ${t('templates.partialFailed') || '项未完成'}):\n${details}`, { timeout: 8000 })
    } else {
      toast.success(t('templates.createSuccess'))
    }

    router.push('/projects')
  } catch (e: any) {
    toast.error(`${t('templates.createFailed')}: ${e?.toString() || e}`)
  } finally {
    isCreating.value = false
  }
}

const handleImport = async () => {
  if (!importUrl.value.trim()) return
  if (!isOnline.value) {
    toast.error(t('common.offlineError') || '网络不可用')
    return
  }
  isImporting.value = true
  try {
    await api.importTemplateFromUrl(importUrl.value.trim())
    toast.success(t('templates.importSuccess'))
    showImportDialog.value = false
    importUrl.value = ''
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.importFailed') || 'Import failed'}: ${e?.toString() || e}`)
  } finally {
    isImporting.value = false
  }
}

const handleDelete = async () => {
  try {
    await api.deleteHubTemplate(deleteTargetId.value)
    toast.success(t('templates.deleteSuccess') || t('templates.saveSuccess'))
    await loadTemplates()
  } catch (e: any) {
    toast.error(`Delete failed: ${e?.toString() || e}`)
  }
  showDeleteConfirm.value = false

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 15 |
================================================================================
}

const handleGenerateFromProject = async () => {
  if (!generateProjectId.value || !generateTemplateName.value.trim()) return
  isGenerating.value = true
  try {
    await api.generateTemplateFromProject(generateProjectId.value, generateTemplateName.value.trim(), generateCategory.value)
    toast.success(t('templates.generateSuccess') || '模板生成成功')
    showGenerateFromProjectDialog.value = false
    generateProjectId.value = ''
    generateTemplateName.value = ''
    generateCategory.value = 'Custom'
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.generateFailed') || '生成失败'}: ${e?.toString() || e}`)
  } finally {
    isGenerating.value = false
  }
}

const progressPercent = computed(() => {
  if (!createProgress.value) return 0
  return Math.round(createProgress.value.progress * 100)
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="shrink-0 px-6 pt-6 pb-4">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('templates.title') }}</h1>
          <p class="text-sm text-gray-500 dark:text-content-muted mt-1">{{ t('templates.subtitle') }}</p>
        </div>
        <button
          @click="showImportDialog = true"
          class="px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
        >
          {{ t('templates.importUrl') }}
        </button>
        <button
          @click="showGenerateFromProjectDialog = true"
          class="px-4 py-2 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
        >
          {{ t('templates.generateFromProject') || '从项目生成' }}
        </button>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex-1 relative">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 16 |
================================================================================
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('projects.search') || 'Search...'"
            class="w-full pl-10 pr-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 focus:border-transparent outline-none"
          />
        </div>
        <div class="flex gap-1 flex-wrap">
          <button
            v-for="cat in categories"
            :key="cat"
            @click="categoryFilter = cat"
            :class="[
              'px-3 py-1.5 text-xs font-medium rounded-full transition-colors',
              categoryFilter === cat
                ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
                : 'bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary hover:bg-gray-200 dark:hover:bg-surface-border'
            ]"
          >
            {{ categoryIcon(cat) }} {{ cat === 'all' ? t('templates.category.all') : t(`templates.category.${cat}`) }}
          </button>
        </div>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <SkeletonList v-if="isLoading" :count="4" />

      <EmptyState
        v-else-if="filteredTemplates.length === 0"
        :title="t('templates.empty')"
        :description="t('templates.emptyDesc')"
        icon="template"
      />

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="tpl in filteredTemplates"
          :key="tpl.template_id"
          class="group relative bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-primary-600 hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden"
          @click="openDetail(tpl)"
        >
          <div class="p-5">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-primary-50 dark:bg-primary-900/20 flex items-center justify-center text-xl">
                  {{ categoryIcon(tpl.category) }}

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 17 |
================================================================================
                </div>
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-content-primary text-sm">{{ tpl.name }}</h3>
                  <p class="text-xs text-gray-500 dark:text-content-muted">{{ t(`templates.category.${tpl.category}`) }}</p>
                </div>
              </div>
              <span
                v-if="tpl.is_builtin"
                class="px-2 py-0.5 text-xs font-medium rounded-full bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400"
              >
                {{ t('templates.builtin') }}
              </span>
            </div>

            <p class="text-xs text-gray-600 dark:text-content-secondary line-clamp-2 mb-3">{{ tpl.description }}</p>

            <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-content-muted mb-4">
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                {{ tpl.godot.version }}
              </span>
              <span v-if="tpl.plugins.length > 0" class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
                {{ tpl.plugins.length }} {{ t('templates.plugins') }}
              </span>
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                {{ tpl.directories.length }} {{ t('templates.directories') }}
              </span>
            </div>

            <div class="flex flex-wrap gap-1.5 mb-4">
              <span
                v-for="tag in tpl.tags.slice(0, 4)"
                :key="tag"
                class="px-2 py-0.5 text-xs rounded-full bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary"
              >
                {{ tag }}
              </span>
            </div>

            <button
              @click.stop="openCreateDialog(tpl)"
              class="w-full py-2 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 18 |
================================================================================
            >
              {{ t('templates.createProject') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Dialog -->
    <Teleport to="body">
      <div v-if="showDetailDialog && selectedTemplate" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="showDetailDialog = false"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto">
          <div class="p-6">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-3">
                <div class="w-12 h-12 rounded-xl bg-primary-50 dark:bg-primary-900/20 flex items-center justify-center text-2xl">
                  {{ categoryIcon(selectedTemplate.category) }}
                </div>
                <div>
                  <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary">{{ selectedTemplate.name }}</h2>
                  <p class="text-sm text-gray-500 dark:text-content-muted">
                    {{ t(`templates.category.${selectedTemplate.category}`) }}
                    <span v-if="selectedTemplate.author"> · {{ selectedTemplate.author }}</span>
                  </p>
                </div>
              </div>
              <button @click="showDetailDialog = false" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-500">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <p class="text-sm text-gray-600 dark:text-content-secondary mb-5">{{ selectedTemplate.description }}</p>

            <div class="grid grid-cols-2 gap-4 mb-5">
              <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.godotVersion') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.godot.version }}{{ selectedTemplate.godot.mono ? ' (Mono)' : '' }}</p>
              </div>
              <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.plugins') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.plugins.length }}</p>
              </div>
            </div>

            <div v-if="selectedTemplate.plugins.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.plugins') }}</h3>
              <div class="space-y-2">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 19 |
================================================================================
                <div
                  v-for="plugin in selectedTemplate.plugins"
                  :key="plugin.name"
                  class="flex items-center justify-between p-2.5 rounded-lg bg-gray-50 dark:bg-surface-layer"
                >
                  <div>
                    <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ plugin.name }}</p>
                    <p class="text-xs text-gray-500 dark:text-content-muted">v{{ plugin.version }} · {{ plugin.source }}</p>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <input
                  id="mobile-support"
                  v-model="enableMobileSupport"
                  type="checkbox"
                  :disabled="isCreating"
                  class="w-4 h-4 rounded border-gray-300 dark:border-surface-border text-primary-600 focus:ring-primary-500"
                />
                <label for="mobile-support" class="text-sm text-gray-700 dark:text-content-secondary cursor-pointer">
                  {{ t('templates.enableMobileSupport') || '添加移动端支持（触摸控件 + 虚拟摇杆）' }}
                </label>
              </div>
            </div>

            <div v-if="selectedTemplate.directories.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.directories') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="dir in selectedTemplate.directories"
                  :key="dir.path"
                  class="px-2.5 py-1 text-xs rounded-lg bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary font-mono"
                >
                  {{ dir.path }}
                </span>
              </div>
            </div>

            <div v-if="selectedTemplate.export_presets.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.exportPresets') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="preset in selectedTemplate.export_presets"
                  :key="preset.name"
                  class="px-2.5 py-1 text-xs rounded-lg bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary"
                >
                  {{ preset.name }} ({{ preset.platform }})
                </span>
              </div>
            </div>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 20 |
================================================================================

            <div class="flex gap-3">
              <button
                @click="showDetailDialog = false; openCreateDialog(selectedTemplate!)"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
              >
                {{ t('templates.createProject') }}
              </button>
              <button
                v-if="!selectedTemplate.is_builtin"
                @click="deleteTargetId = selectedTemplate.template_id; showDeleteConfirm = true"
                class="px-4 py-2.5 text-sm font-medium rounded-lg border border-red-300 dark:border-red-800 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
              >
                {{ t('common.delete') || 'Delete' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Create Project Dialog -->
    <Teleport to="body">
      <div v-if="showCreateDialog && selectedTemplate" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isCreating && (showCreateDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">
              {{ t('templates.createProject') }} — {{ selectedTemplate.name }}
            </h2>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.projectName') || 'Project Name' }}</label>
                <input
                  v-model="createProjectName"
                  type="text"
                  :disabled="isCreating"
                  @input="validateProjectName"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                  :class="{ 'border-red-400 dark:border-red-500': projectNameError }"
                />
                <p v-if="projectNameError" class="mt-1 text-xs text-red-500">{{ projectNameError }}</p>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.targetDir') || 'Target Directory' }}</label>
                <div class="flex gap-2">
                  <input
                    v-model="createTargetDir"
                    type="text"

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 21 |
================================================================================
                    :disabled="isCreating"
                    class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                  />
                  <button
                    @click="selectTargetDir"
                    :disabled="isCreating"
                    class="px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
                  >
                    ...
                  </button>
                </div>
              </div>
            </div>

            <div v-if="createProgress" class="mt-4">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-500 dark:text-content-muted">{{ createProgress.message }}</span>
                <span class="text-xs font-medium text-primary-600 dark:text-primary-400">{{ progressPercent }}%</span>
              </div>
              <div class="w-full bg-gray-200 dark:bg-surface-border rounded-full h-1.5">
                <div
                  class="bg-primary-600 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${progressPercent}%` }"
                ></div>
              </div>
            </div>

            <div class="flex gap-3 mt-6">
              <button
                @click="showCreateDialog = false"
                :disabled="isCreating"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || 'Cancel' }}
              </button>
              <button
                @click="handleCreate"
                :disabled="isCreating || !isValidProjectName || !createTargetDir.trim()"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isCreating ? t('templates.creating') : t('templates.createProject') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Created Success Quick Access -->
    <Teleport to="body">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 22 |
================================================================================
      <div v-if="lastCreatedProjectId" class="fixed bottom-6 right-6 z-50 animate-fade-in">
        <div class="bg-green-600 text-white rounded-xl shadow-lg px-4 py-3 flex items-center gap-3">
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm font-medium">{{ t('templates.createSuccess') }}</span>
          <button
            @click="lastCreatedProjectId = ''"
            class="ml-1 text-green-200 hover:text-white transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </Teleport>

    <!-- Import URL Dialog -->
    <Teleport to="body">
      <div v-if="showImportDialog" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isImporting && (showImportDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">{{ t('templates.importUrl') }}</h2>
            <input
              v-model="importUrl"
              type="url"
              :placeholder="t('templates.importUrlPlaceholder')"
              :disabled="isImporting"
              class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
            />
            <div class="flex gap-3 mt-6">
              <button
                @click="showImportDialog = false"
                :disabled="isImporting"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || 'Cancel' }}
              </button>
              <button
                @click="handleImport"
                :disabled="isImporting || !importUrl.trim() || !isOnline"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isImporting ? '...' : !isOnline ? (t('common.offlineImportTip') || '离线无法导入') : (t('common.import') || '导入') }}
              </button>
            </div>
          </div>
        </div>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 23 |
================================================================================
      </div>

      <div v-if="showGenerateFromProjectDialog" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isGenerating && (showGenerateFromProjectDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">{{ t('templates.generateFromProject') || '从项目生成模板' }}</h2>
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.selectProject') || '选择项目' }}</label>
                <select
                  v-model="generateProjectId"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
                >
                  <option value="" disabled>{{ t('templates.selectProjectPlaceholder') || '请选择项目' }}</option>
                  <option v-for="p in projects" :key="p.project_id" :value="p.project_id">{{ p.name }}</option>
                </select>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.templateName') || '模板名称' }}</label>
                <input
                  v-model="generateTemplateName"
                  type="text"
                  :placeholder="t('templates.templateNamePlaceholder') || '输入模板名称'"
                  :disabled="isGenerating"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.category') || '分类' }}</label>
                <select
                  v-model="generateCategory"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
                >
                  <option value="Custom">{{ t('templates.category.Custom') || '自定义' }}</option>
                  <option value="Starter2D">{{ t('templates.category.Starter2D') || '2D入门' }}</option>
                  <option value="Starter3D">{{ t('templates.category.Starter3D') || '3D入门' }}</option>
                  <option value="RPG">{{ t('templates.category.RPG') || 'RPG' }}</option>
                  <option value="Platformer">{{ t('templates.category.Platformer') || '平台跳跃' }}</option>
                  <option value="Multiplayer">{{ t('templates.category.Multiplayer') || '多人游戏' }}</option>
                  <option value="Mobile">{{ t('templates.category.Mobile') || '移动端' }}</option>
                </select>
              </div>
            </div>
            <div class="flex gap-3 mt-6">
              <button
                @click="showGenerateFromProjectDialog = false"
                :disabled="isGenerating"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 24 |
================================================================================
                {{ t('common.cancel') || '取消' }}
              </button>
              <button
                @click="handleGenerateFromProject"
                :disabled="isGenerating || !generateProjectId || !generateTemplateName.trim()"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isGenerating ? '...' : (t('common.generate') || '生成') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('templates.deleteConfirm')"
      :confirm-text="t('common.delete')"
      confirm-color="red"
      @confirm="handleDelete"
    />
  </div>
</template>

// File: src\views\Updates.vue
<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('nav.updates') }}</h1>
      <button @click="store.checkAll()" :disabled="store.isChecking" class="btn-primary">
        {{ store.isChecking ? t('plugins.checkingUpdates') : t('statusbar.checkUpdates') }}
      </button>
    </div>

    <div v-if="store.lastCheckedAt" class="text-sm text-gray-500 dark:text-content-secondary">
      {{ t('statusbar.lastChecked') }} {{ new Date(store.lastCheckedAt).toLocaleString() }}
    </div>

    <div v-if="store.isInstallingApp" class="card">
      <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('updates.updatingApp') }}</h3>
      <div class="w-full bg-gray-200 dark:bg-surface-hover rounded-full h-2.5">
        <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: store.installProgress + '%' }"></div>
      </div>
      <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ store.installMessage }}</p>
    </div>

    <div v-if="store.hasChecked && !store.isChecking" class="card">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('statusbar.appUpdate') }}</h3>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 25 |
================================================================================
      </div>
      <div v-if="store.appUpdate" class="mt-3">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
              {{ t('updates.currentVersion') }} {{ store.appUpdate.current_version }} → {{ t('updates.latestVersion') }} {{ store.appUpdate.latest_version }}
            </p>
            <p v-if="store.appUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              {{ store.appUpdate.release_notes }}
            </p>
          </div>
          <div class="flex items-center gap-2">
            <button @click="showSkipVersionConfirm = true" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-surface-border rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
              {{ t('updates.skipVersion') }}
            </button>
            <button @click="store.installAppUpdate()" :disabled="store.isInstallingApp" class="btn-primary">
              {{ store.isInstallingApp ? t('statusbar.installing') : t('statusbar.update') }}
            </button>
          </div>
        </div>
        <div class="mt-3 pt-3 border-t border-gray-100 dark:border-surface-border">
          <p class="text-xs text-gray-400 dark:text-content-muted">{{ t('updates.offlineUpdateTip') }}</p>
          <a :href="githubReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-1 text-xs text-primary-600 dark:text-primary-400 hover:underline">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
            {{ t('updates.githubRelease') }}
          </a>
          <a :href="giteeReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-1 text-xs text-primary-600 dark:text-primary-400 hover:underline">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
            {{ t('updates.giteeRelease') }}
          </a>
        </div>
      </div>
      <div v-else class="flex items-center gap-2 mt-2 py-2">
        <svg class="w-4 h-4 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <span class="text-sm text-green-600 dark:text-green-400">{{ t('statusbar.upToDate') }}</span>
      </div>
    </div>

    <div v-if="store.hotUpdate && !store.appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('statusbar.hotUpdate') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            {{ t('plugins.version') }} {{ store.hotUpdate.version }} ({{ formatBytes(store.hotUpdate.download_size) }})
          </p>
          <p v-if="store.hotUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ store.hotUpdate.release_notes }}
          </p>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 26 |
================================================================================
        </div>
        <button @click="store.installHotUpdate()" :disabled="store.isInstallingHotUpdate" class="btn-primary">
          {{ store.isInstallingHotUpdate ? t('statusbar.installing') : t('statusbar.installHotUpdate') }}
        </button>
      </div>
      <div v-if="store.isInstallingHotUpdate" class="mt-3">
        <div class="w-full bg-gray-200 dark:bg-surface-hover rounded-full h-2.5">
          <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: store.hotUpdateProgress + '%' }"></div>
        </div>
        <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ store.hotUpdateMessage }}</p>
      </div>
    </div>

    <div v-if="store.appUpdate && store.hotUpdate" class="card bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
      <p class="text-sm text-blue-700 dark:text-blue-300">
        {{ t('updates.bothUpdatesTip') }}
      </p>
    </div>

    <div v-if="store.hasChecked && !store.isChecking" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('statusbar.plugins') }}
        </h3>
        <button v-if="store.pluginUpdates.length > 0" @click="store.batchUpdateAllPlugins()" :disabled="store.isUpdatingPlugins" class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">
          {{ store.isUpdatingPlugins ? t('statusbar.installing') : t('statusbar.updateAll') }}
        </button>
      </div>
      <div v-if="store.pluginUpdates.length > 0" class="space-y-3">
        <div v-for="update in store.pluginUpdates" :key="update.plugin_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-surface-border last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.plugin_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
            </div>
          </div>
          <button @click="store.updateSinglePlugin(update.plugin_id)" class="px-3 py-1 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            {{ t('statusbar.update') }}
          </button>
        </div>
      </div>
      <div v-else class="flex items-center gap-2 py-2">
        <svg class="w-4 h-4 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <span class="text-sm text-green-600 dark:text-green-400">{{ t('statusbar.upToDate') }}</span>
      </div>
    </div>

    <div v-if="store.hasChecked && !store.isChecking" class="card">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 27 |
================================================================================
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('statusbar.engine') }} {{ t('statusbar.update') }}
        </h3>
        <router-link v-if="store.engineUpdates.length > 0" to="/engines" class="px-3 py-1.5 text-sm border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20">
          {{ t('updates.goToEngines') }}
        </router-link>
      </div>
      <div v-if="store.engineUpdates.length > 0" class="space-y-3">
        <div v-for="update in store.engineUpdates" :key="update.engine_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-surface-border last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.engine_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
              <span v-if="update.is_major_update" class="ml-2 px-1.5 py-0.5 text-xs bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400 rounded">{{ t('statusbar.majorUpdate') }}</span>
            </div>
          </div>
          <a :href="update.download_url" target="_blank" class="px-3 py-1 text-sm border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20">
            {{ t('updates.download') }}
          </a>
        </div>
      </div>
      <div v-else class="flex items-center gap-2 py-2">
        <svg class="w-4 h-4 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <span class="text-sm text-green-600 dark:text-green-400">{{ t('statusbar.upToDate') }}</span>
      </div>
    </div>

    <div v-if="store.currentHotUpdateVersion" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary">{{ t('updates.currentHotUpdateVersion') }} {{ store.currentHotUpdateVersion }}</h3>
        </div>
        <button @click="showRollbackConfirm = true" class="px-3 py-1.5 text-sm border border-red-300 dark:border-red-700 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20">
          {{ t('updates.rollbackHotUpdate') }}
        </button>
      </div>
    </div>

    <div v-if="store.isChecking" class="card text-center py-12">
      <div class="animate-spin w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full mx-auto mb-3"></div>
      <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('updates.checking') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.checkingDesc') }}</p>
    </div>

    <template v-else>
      <div v-if="!store.lastCheckedAt && !store.hasChecked" class="card text-center py-12">
        <svg class="mx-auto h-12 w-12 text-gray-400 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 28 |
================================================================================
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('updates.notCheckedYet') }}</h3>
        <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.notCheckedYetDesc') }}</p>
      </div>

      <div v-if="!store.appUpdate && store.pluginUpdates.length === 0 && store.engineUpdates.length === 0 && !store.hotUpdate && store.lastCheckedAt" class="card text-center py-12">
        <svg class="mx-auto h-12 w-12 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('statusbar.everythingUpToDate') }}</h3>
        <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.allUpToDateDesc') }}</p>
        <a :href="githubReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-3 text-xs text-primary-600 dark:text-primary-400 hover:underline">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
          {{ t('updates.githubRelease') }}
        </a>
        <a :href="giteeReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-3 text-xs text-primary-600 dark:text-primary-400 hover:underline">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
          {{ t('updates.giteeRelease') }}
        </a>
      </div>

      <div v-if="store.checkError && !store.lastCheckedAt" class="card text-center py-12">
        <svg class="mx-auto h-12 w-12 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('updates.checkFailed', { error: '' }) }}</h3>
        <p class="mt-1 text-sm text-red-500 dark:text-red-400">{{ store.checkError }}</p>
        <button @click="store.checkAll()" class="mt-3 px-4 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
          {{ t('statusbar.checkUpdates') }}
        </button>
      </div>
    </template>

    <div v-if="store.updateHistory.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('updates.updateHistory') }} ({{ store.updateHistory.length }})
        </h3>
        <button @click="showClearHistoryConfirm = true" class="px-3 py-1 text-xs border border-gray-300 dark:border-surface-border rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
          {{ t('updates.clearHistory') }}
        </button>
      </div>
      <div class="space-y-2 max-h-80 overflow-y-auto">
        <div v-for="entry in store.updateHistory" :key="entry.id" class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-surface-border last:border-0">
          <div class="flex items-center gap-2">
            <span class="shrink-0 w-5 h-5 flex items-center justify-center rounded text-xs"
              :class="updateTypeClass(entry.update_type)">
              <svg v-if="entry.update_type === 'app'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg>
              <svg v-else-if="entry.update_type === 'plugin'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" /></svg>

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 29 |
================================================================================
              <svg v-else-if="entry.update_type === 'engine'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
              <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
            </span>
            <div>
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ entry.target_name }}</span>
              <span class="text-xs text-gray-500 dark:text-content-secondary ml-2">
                {{ entry.from_version }} → {{ entry.to_version }}
              </span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs px-1.5 py-0.5 rounded"
              :class="entry.status === 'success' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : entry.status === 'rollback' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'">
              {{ entry.status === 'success' ? t('settings.logs.success') : entry.status === 'rollback' ? t('updates.rollbackHotUpdate') : t('updates.failed') }}
            </span>
            <span class="text-xs text-gray-400 dark:text-content-secondary">
              {{ new Date(entry.applied_at).toLocaleDateString() }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="store.lastCheckedAt && store.updateHistory.length === 0" class="card text-center py-8">
      <p class="text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.noHistory') }}</p>
    </div>

    <ConfirmDialog
      v-model="showRollbackConfirm"
      :title="t('updates.rollbackHotUpdate')"
      :description="t('updates.rollbackConfirmDesc')"
      :confirm-text="t('updates.rollbackHotUpdate')"
      confirm-color="red"
      @confirm="store.rollbackHotUpdate()"
    />

    <ConfirmDialog
      v-model="showClearHistoryConfirm"
      :title="t('updates.clearHistory')"
      :description="t('updates.clearHistoryConfirmDesc')"
      :confirm-text="t('updates.clearHistory')"
      confirm-color="orange"
      @confirm="store.clearHistory()"
    />

    <ConfirmDialog
      v-model="showSkipVersionConfirm"
      :title="t('updates.skipVersion')"
      :description="t('updates.skipVersionConfirmDesc')"
      :confirm-text="t('updates.skipVersion')"

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 30 |
================================================================================
      confirm-color="orange"
      @confirm="store.skipAppVersion()"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUpdateStore } from '@/stores/update'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const { t } = useI18n()
const store = useUpdateStore()

const showRollbackConfirm = ref(false)
const showClearHistoryConfirm = ref(false)
const showSkipVersionConfirm = ref(false)

const githubReleaseUrl = '<url>'
const giteeReleaseUrl = '<url>'

function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function updateTypeClass(type: string): string {
  switch (type) {
    case 'app': return 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400'
    case 'plugin': return 'bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400'
    case 'engine': return 'bg-amber-100 text-amber-600 dark:bg-amber-900/30 dark:text-amber-400'
    case 'hot': return 'bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400'
    default: return 'bg-gray-100 text-gray-600 dark:bg-surface-hover dark:text-content-muted'
  }
}

onMounted(async () => {
  await store.initListeners()
  if (!store.lastCheckedAt) {
    store.checkAll()
  }
})

onUnmounted(() => {
  store.cleanupListeners()
})
</script>



