import { api } from '@/api'
import { useToast } from '@/composables/useToast'

export function useFileManager() {
  const toast = useToast()

  const openInFileManager = async (path: string, errorKey?: string) => {
    try {
      await api.openInFileManager(path)
    } catch (error) {
      toast.error(errorKey || String(error))
    }
  }

  return { openInFileManager }
}
