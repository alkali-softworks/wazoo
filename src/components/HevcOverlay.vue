<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { usePlayerStore } from '@/stores/player'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  id: number
  src: string
  codec: string
  title: string
  initialTime: number
}>()

const emit = defineEmits<{
  'found-converted': [newPath: string]
}>()

const playerStore = usePlayerStore()
const { t } = useI18n()

const isConverting = ref(false)
const countdown = ref(5)
const countdownInterval = ref<ReturnType<typeof setInterval> | null>(null)
const conversionOutput = ref('')
const hasReachedLimit = ref(false)
const totalDuration = ref(0)
const progressPercent = ref(0)

const cleanPath = computed(() => props.src.replace(/^file:\/\//, ''))

const startCountdown = () => {
  countdown.value = 5
  stopCountdown()
  countdownInterval.value = setInterval(() => {
    if (countdown.value > 0) {
      countdown.value--
    } else {
      stopCountdown()
      playerStore.playNextVideo(props.id)
      // If we're still here, it means we didn't actually navigate
      // (likely only one video in the list)
      hasReachedLimit.value = true
    }
  }, 1000)
}

const stopCountdown = () => {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value)
    countdownInterval.value = null
  }
}

const handleConvert = async () => {
  stopCountdown()
  isConverting.value = true
  conversionOutput.value = t('player.starting_conversion', 'Starting conversion...')
  
  try {
    // We send the clean path to FFmpeg
    const result = await window.electron.invoke('convert-video', cleanPath.value)
    if (result.success) {
      // Conversion complete, switch to the new file
      emit('found-converted', `file://${result.newPath}`)
    } else {
      console.error('Conversion failed:', result.error)
      isConverting.value = false
      startCountdown() // Restart countdown if failed? Or stay?
    }
  } catch (err) {
    console.error('Conversion error:', err)
    isConverting.value = false
  }
}

// Listen for progress
const progressListener = (data: { output: string, filePath: string }) => {
  if (data.filePath === cleanPath.value) {
    const lines = data.output.split('\n').filter(l => l.trim())
    if (lines.length > 0) {
      const lastLine = lines[lines.length - 1]
      conversionOutput.value = lastLine

      // Parse time=HH:MM:SS.ms
      const timeMatch = lastLine.match(/time=(\d+):(\d+):(\d+\.\d+)/)
      if (timeMatch && totalDuration.value > 0) {
        const hours = parseInt(timeMatch[1])
        const minutes = parseInt(timeMatch[2])
        const seconds = parseFloat(timeMatch[3])
        const currentTime = (hours * 3600) + (minutes * 60) + seconds
        progressPercent.value = Math.min(100, Math.round((currentTime / totalDuration.value) * 100))
      }
    }
  }
}

onMounted(async () => {
  // Get duration for progress bar
  totalDuration.value = await window.electron.invoke('get-video-duration', cleanPath.value)
  
  startCountdown()
  window.electron.on('convert-progress', progressListener)
})

onUnmounted(() => {
  stopCountdown()
  window.electron.off('convert-progress')
})
</script>

<template>
  <div class="hevc-overlay">
    <div class="hevc-content">
      <div class="hevc-icon">
        <svg viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
          <line x1="12" y1="9" x2="12" y2="13"></line>
          <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
      </div>
      <h2 class="video-title">{{ title }}</h2>
      <h3>{{ t('player.hevc_title', { codec }) }}</h3>
      <p>{{ t('player.hevc_description') }}</p>
      
      <div v-if="isConverting" class="conversion-status">
        <div class="progress-container">
          <div class="progress-bar" :style="{ width: progressPercent + '%' }"></div>
          <span class="progress-label">{{ progressPercent }}%</span>
        </div>
        <p class="conversion-text">{{ conversionOutput }}</p>
      </div>
      <div v-else class="hevc-actions">
        <button class="convert-btn" @click="handleConvert">
          {{ t('player.convert_now') }}
        </button>
        <button class="skip-btn" @click="playerStore.playNextVideo(id)" :disabled="hasReachedLimit">
          <template v-if="hasReachedLimit">
            {{ t('player.end_of_playlist') }}
          </template>
          <template v-else>
            {{ t('player.skip_to_next', { countdown }) }}
          </template>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hevc-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  text-align: center;
  padding: 20px;
}

.hevc-content {
  max-width: 400px;
  width: 100%;
}

.hevc-icon {
  color: #fbbf24;
  margin-bottom: 20px;
}

.hevc-content h2 {
  color: white;
  margin-bottom: 5px;
  font-size: 1.2rem;
  font-weight: normal;
  opacity: 0.8;
}

.hevc-content h3 {
  color: white;
  margin-bottom: 15px;
  font-size: 1.5rem;
}

.hevc-content p {
  color: #94a3b8;
  margin-bottom: 25px;
  line-height: 1.5;
}

.hevc-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.convert-btn {
  background: #10b981;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-weight: bold;
  cursor: pointer;
  transition: background 0.2s;
}

.convert-btn:hover {
  background: #059669;
}



.skip-btn {
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.2);
  padding: 10px 24px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.skip-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.conversion-status {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
}

.progress-container {
  width: 100%;
  height: 12px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  overflow: hidden;
  position: relative;
  margin-bottom: 10px;
}

.progress-bar {
  height: 100%;
  background: #10b981;
  transition: width 0.3s ease;
}

.progress-label {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.65rem;
  font-weight: bold;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.conversion-text {
  font-family: monospace;
  font-size: 0.8rem;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}
</style>
