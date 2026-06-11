import { ref } from 'vue'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import { save } from '@tauri-apps/plugin-dialog'
import type { KeyPair, TemplateManifest, SignatureVerification } from '@/types'

export function useTemplateSigner() {
  const toast = useToast()

  const keypairs = ref<KeyPair[]>([])
  const isLoadingKeypairs = ref(false)
  const isGeneratingKeypair = ref(false)
  const isExporting = ref(false)
  const isImporting = ref(false)

  const loadKeypairs = async () => {
    isLoadingKeypairs.value = true
    try {
      keypairs.value = await api.getStoredKeypairs()
    } catch (e: any) {
      toast.error(`加载密钥对失败: ${e?.toString() || e}`)
    } finally {
      isLoadingKeypairs.value = false
    }
  }

  const generateKeypair = async (name: string): Promise<KeyPair | null> => {
    if (!name.trim()) {
      toast.error('请输入签名者名称')
      return null
    }
    isGeneratingKeypair.value = true
    try {
      const keypair = await api.generateSigningKeypair(name.trim())
      await api.saveKeypair(keypair)
      await loadKeypairs()
      toast.success('密钥对生成成功')
      return keypair
    } catch (e: any) {
      toast.error(`生成密钥对失败: ${e?.toString() || e}`)
      return null
    } finally {
      isGeneratingKeypair.value = false
    }
  }

  const deleteKeypair = async (publicKey: string) => {
    try {
      await api.deleteKeypair(publicKey)
      await loadKeypairs()
      toast.success('密钥对已删除')
    } catch (e: any) {
      toast.error(`删除密钥对失败: ${e?.toString() || e}`)
    }
  }

  const exportTemplate = async (templateId: string, templateName: string, signerName?: string): Promise<string | null> => {
    isExporting.value = true
    try {
      // Get the export data as base64
      const dataBase64 = await api.exportTemplateSigned(templateId, signerName)

      // Open save dialog
      const filePath = await save({
        defaultPath: `${templateName}.harbor-template`,
        filters: [{
          name: 'Harbor Template',
          extensions: ['harbor-template']
        }]
      })

      if (!filePath) {
        isExporting.value = false
        return null
      }

      // Write the file
      await api.writeTemplateExport(filePath, dataBase64)
      toast.success('模板导出成功')
      return filePath
    } catch (e: any) {
      toast.error(`导出模板失败: ${e?.toString() || e}`)
      return null
    } finally {
      isExporting.value = false
    }
  }

  const importTemplate = async (filePath: string): Promise<TemplateManifest | null> => {
    isImporting.value = true
    try {
      const manifest = await api.importTemplateFromFile(filePath)
      return manifest
    } catch (e: any) {
      toast.error(`导入模板失败: ${e?.toString() || e}`)
      return null
    } finally {
      isImporting.value = false
    }
  }

  const verifySignature = async (manifest: TemplateManifest): Promise<SignatureVerification | null> => {
    try {
      return await api.verifyTemplateSignature(manifest)
    } catch (e: any) {
      toast.error(`验证签名失败: ${e?.toString() || e}`)
      return null
    }
  }

  return {
    keypairs,
    isLoadingKeypairs,
    isGeneratingKeypair,
    isExporting,
    isImporting,
    loadKeypairs,
    generateKeypair,
    deleteKeypair,
    exportTemplate,
    importTemplate,
    verifySignature,
  }
}
