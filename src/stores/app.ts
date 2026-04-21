import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppState = defineStore('appState', () => {
  const initialLoad = ref(true)
  const query = ref('')
  const isScanning = ref(false)
  const scanProgress = ref(0)
  const lastVideoScanned = ref('')
  const totalFiles = ref(0)

  const setScanProgress = (scanning: boolean, progress: number, total: number, videoName?: string) => {
    isScanning.value = scanning
    scanProgress.value = progress
    totalFiles.value = total
    lastVideoScanned.value = videoName || ''
  }

  const setQuery = (q: string) => {
    query.value = q
  }

  const setInitialLoad = (v: boolean) => {
    initialLoad.value = v
  }

  return {
    isScanning,
    scanProgress,
    totalFiles,
    lastVideoScanned,
    setScanProgress,
    query,
    setQuery,
    initialLoad,
    setInitialLoad
  }
})
