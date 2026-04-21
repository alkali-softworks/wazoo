<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { Slider } from '@/components/ui/slider'
import { Loader2 } from 'lucide-vue-next'
import { useAppState } from '@/stores/app'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'
import { log } from '@/lib/utils'

const props = defineProps<{
  isOpen: boolean;
  onClose: () => void;
}>();

const emit = defineEmits(['scan-start', 'scan-end'])

const appState = useAppState()
const settingsStore = useSettingsStore()
const { t } = useI18n()
const videoCount = ref(0)
let localScanId = 0

const handleKeyDown = (e: KeyboardEvent) => {
  e.stopPropagation();
  if (e.key === 'Escape' || e.key === 'h') {
    props.onClose();
  }
}

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    window.addEventListener('keydown', handleKeyDown);
  } else {
    window.removeEventListener('keydown', handleKeyDown);
  }
});

const opacity = ref([settingsStore.windowOpacity])

function setWindowOpacity(opacity: number) {
  window.electron.invoke('set-window-opacity', opacity)
}

function handleOpacityChange(values: number[]) {
  const value = values[0]
  opacity.value = [value]
  setWindowOpacity(value)
  settingsStore.setWindowOpacity(value)
}

async function addFolder() {
  if (window.electron) {
    const result = await window.electron.invoke('select-folder')
    if (result) {
      await settingsStore.addMediaFolder(result)
      log('starting scan...')
      await scanFolders()
    }
  }
}

async function scanFolders() {
  const scanId = Date.now()
  localScanId = scanId
  try {
    appState.setScanProgress(true, 0, 0)
    emit('scan-start')
    const result = await window.electron.invoke('scan-folders', scanId)
    
    if (localScanId !== scanId) return

    if (result.success) {
      const countResult = await window.electron.invoke('get-video-count')
      if (countResult.success) {
        videoCount.value = countResult.count
      }
    } else {
      if (result.error !== 'cancelled') {
        console.error('Error scanning folders:', result.error)
      }
    }
  } catch (error) {
    if (localScanId !== scanId) return
    console.error('Error scanning folders:', error)
  } finally {
    if (localScanId === scanId) {
      appState.setScanProgress(false, 0, 0)
      emit('scan-end')
    }
  }
}

async function removeFolder(folder: string) {
  await settingsStore.removeMediaFolder(folder)
  log('starting scan...')
  await scanFolders()
}

function toggleRunServer(checked: boolean) {
  log('toggleRunServer', checked)
  settingsStore.setRunServer(checked)
}

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {

    window.addEventListener('keydown', handleKeyDown)

    const result = await window.electron.invoke('get-video-count')
    if (result.success) {
      videoCount.value = result.count
    }

  } else {
    // Modal is closed
    window.removeEventListener('keydown', handleKeyDown)
    //window.electron.off('scan-progress')
  }
})


onMounted(() => {
  window.electron.on('scan-progress', (data: { 
      processed: number,
      total: number,
      percent: number,
      scanId: number,
      name?: string
     }
  ) => {
    if (localScanId === data.scanId) {
      appState.setScanProgress(true, data.percent, data.total, data.name)
    }
  })
})

</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click="onClose">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>{{ t('settings.title') }}</h2>
        <button class="close-modal" @click="onClose">✕</button>
      </div>
      <div class="modal-body">

        <!-- Language Settings -->
        <div class="setting-group">
          <label class="setting-label">{{ t('common.settings') }} - Language</label>
          <div class="flex items-center gap-4">
            <select 
              :value="settingsStore.language" 
              @change="(e) => settingsStore.setLanguage((e.target as HTMLSelectElement).value as 'en' | 'es' | 'ja' | 'zh' | 'fr' | 'de' | 'ru' | 'it' | 'pt' | 'ko' | 'hi' | 'bn' | 'ar' | 'id' | 'he' | 'th')"
              class="language-select"
            >
              <option value="en">English</option>
              <option value="es">Español</option>
              <option value="fr">Français</option>
              <option value="de">Deutsch</option>
              <option value="it">Italiano</option>
              <option value="pt">Português</option>
              <option value="ru">Русский (Russian)</option>
              <option value="zh">简体中文 (Chinese)</option>
              <option value="ja">日本語 (Japanese)</option>
              <option value="ko">한국어 (Korean)</option>
              <option value="th">ไทย (Thai)</option>
              <option value="he">עברית (Hebrew)</option>
              <option value="ar">العربية (Arabic)</option>
              <option value="hi">हिन्दी (Hindi)</option>
              <option value="bn">বাংলা (Bengali)</option>
              <option value="id">Bahasa Indonesia</option>
            </select>
          </div>
        </div>

        <!-- Window Opacity -->
        <div class="setting-group">
          <label class="setting-label">{{ t('settings.window_opacity') }}</label>
          <div class="flex items-center gap-4">
            <Slider
              :model-value="opacity"
              @update:model-value="handleOpacityChange"
              :min="0.1"
              :max="1"
              :step="0.01"
              class="w-[100%]"
            />
            <span class="text-sm text-muted-foreground w-12">
              {{ Math.round(opacity[0] * 100) }}%
            </span>
          </div>
        </div>

        <!-- Media Folders -->
        <div class="setting-group">
          <label class="setting-label">{{ t('settings.media_folders') }}</label>

          <div class="folders-list">
            <div v-for="folder in settingsStore.mediaFolders" :key="folder" class="folder-item">
              <span class="folder-path">{{ folder }}</span>
              <button @click="removeFolder(folder)" class="remove-folder-btn">
                ✕
              </button>
            </div>
          </div>

          <div class="folder-controls">
            <button 
              @click="scanFolders" 
              class="add-folder-btn cursor-pointer" 
              :disabled="appState.isScanning"
            >
              <Loader2 
                v-if="appState.isScanning" 
                class="animate-spin mr-2 h-4 w-4 inline-block"
              />
              {{ appState.isScanning 
                ? (appState.totalFiles === 0 ? t('wazoo.listing_files') : t('wazoo.loading_progress', { percent: appState.scanProgress, total: appState.totalFiles.toLocaleString() }))
                : t('settings.scan_folders') 
              }}
            </button>

            <button @click="addFolder" class="add-folder-btn">
              {{ t('settings.add_folder') }}
            </button>
          </div>

          <template v-if="appState.isScanning">
            <div class="text-sm text-muted-foreground text-wrap" style="max-width: 260px; white-space: normal; line-height: 1.4; margin-top: 4px;">
              {{ t('settings.adding_media_notice') }}
            </div>
          </template>
        </div>

        <!-- Video Count -->
        <div class="video-count">
          {{ t('settings.total_videos', { count: videoCount.toLocaleString() }) }}
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100000;
}

.modal-content {
  background: #1f1f1f;
  border-radius: 8px;
  padding: 20px;
  min-width: 300px;
  max-width: 500px;
  color: white;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.5em;
}

.close-modal {
  background: none;
  border: none;
  color: #888;
  font-size: 1.2em;
  cursor: pointer;
  padding: 5px;
}

.close-modal:hover {
  color: white;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: #888;
}

.folder-controls {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.add-folder-btn {
  background: #333;
  border: 1px solid #444;
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.add-folder-btn:hover {
  background: #444;
}

.folders-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.folder-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #2a2a2a;
  padding: 8px 12px;
  border-radius: 4px;
}

.folder-path {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.remove-folder-btn {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px 8px;
}

.remove-folder-btn:hover {
  color: #ff4444;
}

.video-count {
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid #333;
  font-size: 14px;
  color: #888;
  text-align: right;
}

.add-folder-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.add-folder-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.language-select {
  background: #2a2a2a;
  color: white;
  border: 1px solid #3f3f3f;
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 14px;
  outline: none;
  cursor: pointer;
  width: 100%;
}

.language-select:focus {
  border-color: #5f5f5f;
}
</style>