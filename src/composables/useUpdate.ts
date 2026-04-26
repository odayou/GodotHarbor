import { ref, onMounted, onUnmounted } from 'vue'
import { useUpdateStore } from '@/stores/update'
import { useSettingsStore } from '@/stores'
import { api } from '@/api'
import { useI18n } from 'vue-i18n'

export function useUpdate() {
  const store = useUpdateStore()
  const settingsStore = useSettingsStore()
  const { t } = useI18n()
  const appVersion = ref('')

  async function loadAppVersion() {
    try {
      appVersion.value = await api.getAppVersion()
    } catch {
      appVersion.value = '0.1.0'
    }
  }

  async function sendSystemNotification(title: string, body: string) {
    try {
      const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification')
      let permitted = await isPermissionGranted()
      if (!permitted) {
        const permission = await requestPermission()
        permitted = permission === 'granted'
      }
      if (permitted) {
        const { sendNotification } = await import('@tauri-apps/plugin-notification')
        sendNotification({ title, body })
      }
    } catch (e) {
      console.error('Failed to send notification:', e)
    }
  }

  async function checkAndNotify() {
    await store.checkAll()
    if (store.hasAnyUpdate) {
      const parts: string[] = []
      if (store.appUpdate) {
        parts.push(`${t('statusbar.appUpdate')}: v${store.appUpdate.latest_version}`)
      }
      if (store.pluginUpdates.length > 0) {
        parts.push(`${store.pluginUpdates.length} ${t('statusbar.plugins')}`)
      }
      if (store.engineUpdates.length > 0) {
        parts.push(`${store.engineUpdates.length} ${t('statusbar.engine')}`)
      }
      if (store.hotUpdate) {
        parts.push(`${t('statusbar.hotUpdate')}: ${store.hotUpdate.version}`)
      }
      await sendSystemNotification(
        t('statusbar.updateAvailable'),
        `${t('statusbar.updateAvailable')}: ${parts.join(', ')}`
      )
    }
  }

  async function shouldCheckOnStartup(): Promise<boolean> {
    await settingsStore.loadSettings()
    const settings = settingsStore.settings
    const autoCheckApp = settings.auto_check_app_updates !== false
    const autoCheckPlugin = settings.auto_check_plugin_updates !== false
    const autoCheckEngine = settings.auto_check_engine_updates !== false
    return autoCheckApp || autoCheckPlugin || autoCheckEngine
  }

  onMounted(async () => {
    await loadAppVersion()
    await store.initListeners()

    if (await shouldCheckOnStartup()) {
      setTimeout(async () => {
        await checkAndNotify()
      }, 30000)
    }
  })

  onUnmounted(() => {
    store.cleanupListeners()
  })

  return {
    store,
    appVersion,
    checkAndNotify,
    sendSystemNotification,
    loadAppVersion
  }
}
