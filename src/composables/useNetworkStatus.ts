import { ref } from 'vue'

export const isOnline = ref(navigator.onLine)

const goOnline = () => { isOnline.value = true }
const goOffline = () => { isOnline.value = false }

if (typeof window !== 'undefined') {
  window.addEventListener('online', goOnline)
  window.addEventListener('offline', goOffline)
}

export function useNetworkStatus() {
  return { isOnline }
}
