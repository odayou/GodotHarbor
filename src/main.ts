import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import i18n from './locales'
import App from './App.vue'
import './style.css'

const app = createApp(App)

app.config.errorHandler = (err, _instance, info) => {
  console.error('Vue global error:', err, info)
  try {
    const { invoke } = require('@tauri-apps/api/core')
    invoke('log_client_error', {
      source: `vue:errorHandler:${info}`,
      error: String(err)
    }).catch(() => {})
  } catch {}
}

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason)
  event.preventDefault()
})

app.use(createPinia())
app.use(router)
app.use(i18n)

app.mount('#app')
