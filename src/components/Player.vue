<script setup lang="ts">
import { PlayerState, VideoEventPayload, StateChangePayload } from '@/types'
import { ref, reactive, watch, computed, onMounted, onUnmounted, inject, nextTick } from 'vue'
import { usePlayerStore } from '@/stores/player'
import { useI18n } from 'vue-i18n'
import SubtitleDisplay from '@/components/SubtitleDisplay.vue'
import HevcOverlay from '@/components/HevcOverlay.vue'
import { formatTime, formatVideoTitle, formatVideoFolder } from '@/lib/utils'
import apiClient from '@/lib/apiClient'
import { 
  Volume2 as VolumeIcon,
  VolumeX as VolumeOffIcon,
  Play as PlayIcon,
  Pause as PauseIcon,
  SkipForward as NextIcon
} from 'lucide-vue-next'
import { log } from '@/lib/utils'

// Props
const props = defineProps({
  id: {
    type: Number,
    required: true
  },
  src: {
    type: String,
    required: true
  },
  codec: {
    type: String,
    default: ''
  },
  initialTime: {
    type: Number,
    default: 0
  },
  initialVolume: {
    type: Number,
    default: 1
  },
  initialMuted: {
    type: Boolean,
    default: true
  },
  scrollMode: {
    type: Boolean,
    default: true
  },
})

// Emits
const emit = defineEmits<{
  'video-event': [payload: VideoEventPayload]
  'state-change': [payload: StateChangePayload]
}>()

// Core state
const playerStore = usePlayerStore()
const { t } = useI18n()
const showNotice = inject('showNotice') as (message: string) => void

const internalSrc = ref(props.src)
const isResolvingSrc = ref(false)
const isStreaming = ref(false)
const streamStartTime = ref(0)
const realDuration = ref(0)
const isHevc = computed(() => {
  if (isResolvingSrc.value) return false
  if (internalSrc.value !== props.src) return false
  return props.codec?.toLowerCase().includes('hevc')
})

// DOM Refs
const videoRef = ref<HTMLVideoElement | null>(null)
const progressBarRef = ref<HTMLElement | null>(null)
const volumeBarRef = ref<HTMLElement | null>(null)
const playing = ref(false)

// Timeouts
const overlayTimeout = ref<ReturnType<typeof setTimeout> | null>(null)
const borderTimeout = ref<ReturnType<typeof setTimeout> | null>(null)

// Main state object
const state = reactive<PlayerState>({
  videoTime: 0,
  videoDuration: 0,
  videoVolume: 0,
  videoMuted: props.initialMuted,
  isScrubbing: false,
  isVolumeScrubbing: false,
  isOverlayVisible: false,
  subtitleData: '',
  subtitlesEnabled: true,
  hideVideoBorder: false,
  lastVideoTime: 0,
})

// Computed properties
const formattedTitle = computed(() => formatVideoTitle(props.src))
const formattedFolder = computed(() => formatVideoFolder(props.src))
const videoVolumePercent = computed(() => state.videoVolume * 100)
const currentTime = computed(() => formatTime(state.videoTime))
const duration = computed(() => formatTime(state.videoDuration))
const progress = computed(() => {
  if (!state.videoDuration) return 0
  return (state.videoTime / state.videoDuration) * 100
})

const videoMimeType = computed(() => {
  const lowerSrc = internalSrc.value.toLowerCase()
  if (lowerSrc.endsWith('.mp4') || lowerSrc.endsWith('.hevc')) {
    return 'video/mp4'
  }
  if (lowerSrc.endsWith('.webm')) {
    return 'video/webm'
  }
  return '' // Let HTML5 try its best
})

// Utility functions
const safeVideoOperation = (operation: (video: HTMLVideoElement) => void) => {
  if (!videoRef.value) {
    console.warn('Video element not available')
    return
  }
  operation(videoRef.value)
}

const clearTimeouts = () => {
  if (overlayTimeout.value) clearTimeout(overlayTimeout.value)
  if (borderTimeout.value) clearTimeout(borderTimeout.value)
}

// Controller objects
const videoController = {
  play: () => safeVideoOperation(video => video.play()),
  pause: () => safeVideoOperation(video => video.pause()),
  togglePlay: () => {
    safeVideoOperation(video => {
      video.paused ? video.play() : video.pause()
    })
  },
  seek: (seconds: number) => {
    safeVideoOperation(video => {
      if (isStreaming.value) {
        const newTime = state.videoTime + seconds;
        const clampedNewTime = Math.max(0, Math.min(newTime, realDuration.value));
        state.videoTime = clampedNewTime;
        debouncedStreamSeek(clampedNewTime);
      } else {
        video.currentTime = video.currentTime + seconds
      }
      overlayController.show()
    })
  },
  nextFrame: () => {
    safeVideoOperation(video => {
      video.pause()
      video.currentTime = video.currentTime + 1/24
    })
  },
  previousFrame: () => {
    safeVideoOperation(video => {
      video.pause()
      video.currentTime = video.currentTime - 1/24
    })
  },
  toggleFullscreen: () => {
    safeVideoOperation(video => video.requestFullscreen())
  }
}

const volumeController = {
  adjust: (delta: number) => {
    safeVideoOperation(video => {
      const newVolume = Math.max(0, Math.min(1, video.volume + delta))
      video.volume = newVolume
      state.videoVolume = newVolume
      
      // If we are turning volume up, we must explicitly Unmute the state
      if (newVolume > 0.05) {
        state.videoMuted = false
        video.muted = false
      } else {
        video.muted = true
      }
    })
  },
  toggle: () => {
    state.videoMuted = !state.videoMuted
    // Force immediate hardware update
    if (videoRef.value) videoRef.value.muted = state.videoMuted
    showNotice(state.videoMuted ? t('player.muted') : t('player.unmuted'))
  },
  getFormatted: () => {
    if (!videoRef.value) return '0%'
    return `${Math.round(videoRef.value.volume * 100)}%`
  }
}

const overlayController = {
  show: () => {
    state.isOverlayVisible = true
    safeVideoOperation(video => {
      if (video.parentElement) {
        video.parentElement.style.cursor = 'default'
      }
    })
    clearTimeout(overlayTimeout.value)
    overlayTimeout.value = setTimeout(() => {
      state.isOverlayVisible = false
      safeVideoOperation(video => {
        if (video.parentElement) {
          video.parentElement.style.cursor = 'none'
        }
      })
    }, 2000)
  }
}

const handleFoundConverted = (newPath: string) => {
  log('Switching to converted video:', newPath)
  internalSrc.value = newPath
  
  nextTick(() => {
    safeVideoOperation(v => {
      v.load()
      if (playing.value) v.play().catch(() => {})
    })
  })
}

let seekTimeout: ReturnType<typeof setTimeout> | null = null

const debouncedStreamSeek = (newTime: number) => {
  if (seekTimeout) clearTimeout(seekTimeout)
  seekTimeout = setTimeout(() => {
    const cleanPath = props.src.replace(/^file:\/\//, '')
    const encodedPath = encodeURIComponent(cleanPath)
    streamStartTime.value = newTime
    const streamUrl = `${apiClient.baseUrl()}/stream?path=${encodedPath}&start=${newTime}&t=${Date.now()}`
    
    log('Debounced seeking stream to:', newTime)
    internalSrc.value = streamUrl
    nextTick(() => {
      safeVideoOperation(v => {
        v.load()
        if (playing.value) v.play().catch(() => {})
      })
    })
  }, 500)
}

const handleStartStream = async (streamUrl: string) => {
  log('Switching to streaming video:', streamUrl)
  
  isStreaming.value = true
  streamStartTime.value = props.initialTime || 0
  
  const cleanPath = props.src.replace(/^file:\/\//, '')
  realDuration.value = await window.electron.invoke('get-video-duration', cleanPath)
  state.videoDuration = realDuration.value
  
  internalSrc.value = streamUrl
  
  nextTick(() => {
    safeVideoOperation(v => {
      v.load()
      if (playing.value) v.play().catch(() => {})
    })
  })
}

const checkConverted = async (path: string) => {
  try {
    const clean = path.replace(/^file:\/\//, '')
    const pathParts = clean.match(/^(.*)\.([^.]+)$/)
    if (!pathParts) return null

    const dirAndName = pathParts[1]
    const wazooPath = `${dirAndName}_wazoo.mkv`
    
    const exists = await window.electron.invoke('file-exists', wazooPath)
    if (exists) return `file://${wazooPath}`
  } catch (err) {
    console.error('Error checking for version:', err)
  }
  return null
}

const resolveSource = async (src: string) => {
  log('Resolving source:', src)
  isResolvingSrc.value = true
  isStreaming.value = false
  streamStartTime.value = 0
  realDuration.value = 0
  
  try {
    const converted = await checkConverted(src)
    if (converted) {
      log('Found converted version:', converted)
      internalSrc.value = converted
    } else {
      internalSrc.value = src
    }
  } catch (err) {
    console.error('Failed to resolve source:', err)
    internalSrc.value = src
  } finally {
    isResolvingSrc.value = false
  }

  nextTick(() => {
    safeVideoOperation(video => {
      video.load()
      video.muted = state.videoMuted 
      video.currentTime = props.initialTime
      subtitleController.load()
      if (playing.value) {
        video.play().catch(err => {
          log('Auto-play failed after resolution:', err)
        })
      }
    })
  })
}

// Event handlers
const handleVideoEvent = (event: Event) => {
  // if (event.type === 'volumechange' && videoRef.value) {
  //   state.videoMuted = videoRef.value.muted
  // }

  state.videoVolume = videoRef.value?.volume || 0
  emit('video-event', { event, playerId: props.id, videoRef: videoRef.value })
}

const handleTimeUpdate = () => {
  safeVideoOperation(video => {
    if (state.isScrubbing && isStreaming.value) {
      // Don't let the background stream fight with the visual scrubber while dragging!
      return
    }
    
    if (isStreaming.value) {
      state.videoTime = streamStartTime.value + video.currentTime
      state.videoDuration = realDuration.value
    } else {
      state.videoTime = video.currentTime
      state.videoDuration = video.duration
    }
    //emitStateChange()
  })
}

// Focus management
const handleClick = () => {
  playerStore.setFocusedPlayer(props.id)
}

const handleFocusChange = () => {
  clearTimeout(borderTimeout.value)
  overlayController.show()
  state.hideVideoBorder = false

  borderTimeout.value = setTimeout(() => {
    state.hideVideoBorder = true
  }, 300)
}

// Progress bar controls
const progressBarController = {
  handleMouseDown: (event: MouseEvent) => {
    playerStore.setFocusedPlayer(props.id)
    state.isScrubbing = true
    progressBarController.updateProgress(event)
    document.addEventListener('mousemove', progressBarController.handleMouseMove)
    document.addEventListener('mouseup', progressBarController.handleMouseUp)
  },

  handleMouseMove: (event: MouseEvent) => {
    if (state.isScrubbing) {
      progressBarController.updateProgress(event)
    }
  },

  handleMouseUp: () => {
    state.isScrubbing = false
    document.removeEventListener('mousemove', progressBarController.handleMouseMove)
    document.removeEventListener('mouseup', progressBarController.handleMouseUp)
  },

  updateProgress: (event: MouseEvent) => {
    if (!progressBarRef.value) return
    
    const rect = progressBarRef.value.getBoundingClientRect()
    const clickPosition = Math.max(0, Math.min(event.clientX - rect.left, rect.width))
    const percent = (clickPosition / rect.width) * 100

    safeVideoOperation(video => {
      if (isStreaming.value) {
        const newTime = (percent / 100) * realDuration.value;
        state.videoTime = newTime; // visually simulate progress instantly
        debouncedStreamSeek(newTime);
      } else {
        if (video.duration) {
          video.currentTime = (percent / 100) * video.duration
        }
      }
    })
  }
}

// Volume bar controls
const volumeBarController = {
  handleMouseDown: (event: MouseEvent) => {
    playerStore.setFocusedPlayer(props.id)
    state.isVolumeScrubbing = true
    volumeBarController.updateVolume(event)
    document.addEventListener('mousemove', volumeBarController.handleMouseMove)
    document.addEventListener('mouseup', volumeBarController.handleMouseUp)
  },

  handleMouseMove: (event: MouseEvent) => {
    if (state.isVolumeScrubbing) {
      volumeBarController.updateVolume(event)
    }
  },

  handleMouseUp: () => {
    state.isVolumeScrubbing = false
    document.removeEventListener('mousemove', volumeBarController.handleMouseMove)
    document.removeEventListener('mouseup', volumeBarController.handleMouseUp)
  },

  updateVolume: (event: MouseEvent) => {
    if (!volumeBarRef.value) return
    
    const rect = volumeBarRef.value.getBoundingClientRect()
    const clickPosition = Math.max(0, Math.min(event.clientX - rect.left, rect.width))
    const percent = (clickPosition / rect.width) * 100

    safeVideoOperation(video => {
      const newVolume = Math.max(0, Math.min(1, percent / 100))
      video.volume = newVolume
      state.videoVolume = newVolume
      video.muted = newVolume < 0.1
    })
  }
}

// Subtitle management
const subtitleController = {
  async load() {
    try {
      const srtFileName = props.src.replace(/\.[^/.]+$/, ".srt")
      const cleanPath = props.src.replace(/^file:\/\//, '')
      const cleanSrtPath = srtFileName.replace(/^file:\/\//, '')

      // 1. Check if the .srt file exists on disk
      const exists = await window.electron.invoke('file-exists', cleanSrtPath)
      
      if (!exists) {
        // 2. If not, check if the video has internal subtitles
        const { hasSubtitles } = await window.electron.invoke('check-video-subs', cleanPath)
        
        if (hasSubtitles) {
          showNotice(t('player.starting_subtitle_extraction'))
          const extractRes = await window.electron.invoke('extract-subtitles', cleanPath)
          
          if (!extractRes.success) {
            log('Extraction failed:', extractRes.error)
            state.subtitleData = ''
            return
          }
          log('Subtitles extracted to:', extractRes.outputPath)
        } else {
          state.subtitleData = ''
          return
        }
      }

      // 3. Load the (newly extracted or already existing) subtitle file
      const response = await fetch(srtFileName)
      
      if (!response.ok) {
        state.subtitleData = ''
        return
      }
      
      state.subtitleData = await response.text()

    } catch (error) {
      log('Error loading subtitles:', error)
      state.subtitleData = ''
    }
  },

  toggle() {
    if (!state.subtitleData) return
    state.subtitlesEnabled = !state.subtitlesEnabled
    showNotice(state.subtitlesEnabled ? t('player.subtitles_enabled') : t('player.subtitles_disabled'))
  }
}

// Lifecycle hooks
onMounted(() => {
  if (videoRef.value) {
    videoRef.value.muted = state.videoMuted
  }
  resolveSource(props.src)
})

onUnmounted(() => {
  clearTimeouts()
  document.removeEventListener('mousemove', volumeBarController.handleMouseMove)
  document.removeEventListener('mouseup', volumeBarController.handleMouseUp)
  document.removeEventListener('mousemove', progressBarController.handleMouseMove)
  document.removeEventListener('mouseup', progressBarController.handleMouseUp)
})

// Watchers
watch(() => props.src, (newSrc) => {
  resolveSource(newSrc)
})

watch(() => playerStore.isFocusedPlayer(props.id), (isFocused) => {
  if (isFocused) {
    if (props.scrollMode) {
      state.hideVideoBorder = true
    } else {
      handleFocusChange()
    }
  } else {
    state.hideVideoBorder = false
    clearTimeout(borderTimeout.value)
  }
})

// Expose public methods
defineExpose({
  showOverlay: overlayController.show,
  seek: videoController.seek,
  adjustVolume: volumeController.adjust,
  getVolume: volumeController.getFormatted,
  setInternalMute: (muted: boolean) => {
    state.videoMuted = muted
    safeVideoOperation(video => {
      video.muted = muted
    })
  },

  setVolume: (value: number) => {
    safeVideoOperation(video => {
      const clamped = Math.max(0, Math.min(1, value))
      video.volume = clamped
      state.videoVolume = clamped

      // Priority Logic:
      // 1. If user (or global state) explicitly wants mute, FORCE mute.
      // 2. Else, only auto-mute if volume is effectively zero (cleanup).
      if (state.videoMuted) {
        video.muted = true
      } else {
        video.muted = clamped < 0.05
      }
    })
  },
  toggleMute: volumeController.toggle,
  togglePlay: videoController.togglePlay,
  toggleFullscreen: videoController.toggleFullscreen,
  nextFrame: videoController.nextFrame,
  previousFrame: videoController.previousFrame,
  getHeight: () => videoRef.value?.offsetHeight || 0,
  setPlaying: (value: boolean) => playing.value = value,
  getPlaying: () => playing.value,
  toggleSubtitles: subtitleController.toggle,
  getCurrentTime: () => state.videoTime,
  getDuration: () => state.videoDuration,
  getMuted: () => state.videoMuted,
  getSrc: () => props.src,
  getVideoObject: () => videoRef.value,
  getLastTime: () => state.videoTime,
  setLastTime: (n:number) => state.videoTime = n,
})
</script>

<template>
  <div 
    class="player-container"
    :class="{ 
      'player-focused': playerStore.isFocusedPlayer(id),
      'remove-border': state.hideVideoBorder 
    }"
    @click="handleClick"
    @mousemove="overlayController.show"
  >
    <!-- Title Overlay -->
    <div 
      class="video-overlay-title" 
      :class="{ 'overlay-visible': state.isOverlayVisible }"
    >
      <h3><span>{{ formattedTitle }}</span></h3>
    </div>

    <!-- Controls Overlay -->
    <div 
      class="video-progress-overlay" 
      :class="{ 'overlay-visible': state.isOverlayVisible }"
    >
      <div class="controls-wrapper">
        <!-- Volume Controls -->
        <div class="volume-control">
          <button 
            class="subtitle-toggle" 
            :class="{ 'disabled': !state.subtitleData }" 
            @click.stop="subtitleController.toggle"
          >
            {{ state.subtitlesEnabled ? 'CC' : 'cc' }}
          </button>
          
          <VolumeOffIcon 
            v-if="state.videoMuted" 
            class="volume-icon" 
            @click="volumeController.toggle" 
          />
          <VolumeIcon 
            v-else 
            class="volume-icon" 
            @click="volumeController.toggle" 
          />
          <div 
            class="volume-bar" 
            ref="volumeBarRef" 
            @mousedown="volumeBarController.handleMouseDown" 
            @click.stop
          >
            <div 
              class="volume-bar-fill" 
              :style="{ width: `${videoVolumePercent}%` }"
            />
          </div>
        </div>

        <!-- Playback Controls -->
        <div class="playback-control">
          <PlayIcon 
            v-if="videoRef?.paused" 
            class="control-icon" 
            @click.stop="videoController.togglePlay" 
          />
          <PauseIcon 
            v-else 
            class="control-icon" 
            @click.stop="videoController.togglePlay" 
          />
          <NextIcon 
            class="control-icon" 
            @click.stop="playerStore.playNextVideo(props.id)" 
          />
        </div>
      </div>

       <!-- Progress Bar -->
      <div 
        ref="progressBarRef" 
        class="progress-bar" 
        @mousedown="progressBarController.handleMouseDown" 
        @click.stop
      >
        <div 
          class="progress-bar-fill" 
          :style="{ width: `${progress}%` }"
        />
        <span class="time-display">{{ currentTime }} / {{ duration }}</span>
      </div>
    </div>

    <!-- Subtitles -->
    <SubtitleDisplay
      v-if="state.subtitlesEnabled"
      :subtitle-content="state.subtitleData"
      :current-time="state.videoTime"
    />

    <!-- Video Element -->
    <video 
      ref="videoRef"
      :id="`player-${id}`"
      autoplay
      :muted="state.videoMuted"
      @timeupdate="handleTimeUpdate"
      @volumechange="handleVideoEvent"
      @play="handleVideoEvent"
      @pause="handleVideoEvent"
      @ended="handleVideoEvent"
      @loadstart="handleVideoEvent"
      @loadeddata="handleVideoEvent"
      @loadedmetadata="handleVideoEvent"
      @waiting="handleVideoEvent"
      @playing="handleVideoEvent"
      @canplay="handleVideoEvent"
      @canplaythrough="handleVideoEvent"
      @seeking="handleVideoEvent"
      @seeked="handleVideoEvent"
      @stalled="handleVideoEvent"
      @suspend="handleVideoEvent"
      @error="handleVideoEvent"
    >
      <source :src="internalSrc" :type="videoMimeType" />
    </video>

    <HevcOverlay 
      v-if="isHevc" 
      :key="props.src"
      :id="id" 
      :src="props.src" 
      :codec="codec"
      :title="formattedTitle"
      :initial-time="props.initialTime"
      @found-converted="handleFoundConverted"
      @start-stream="handleStartStream"
    />
  </div>
</template>

<style scoped>
/* Container styles */
.player-container {
  position: relative;
  width: 100%;
  cursor: none;
}

.player-focused {
  border: 2px solid #42b883;
}

.player-focused.remove-border {
  border: 0;
}

/* Video element */
video {
  width: 100%;
  height: auto;
}

/* Title overlay */
.video-overlay-title {
  position: absolute;
  top: 16%;
  left: 0;
  display: inline-block;
  padding: 6px 10px;
  background: rgba(0, 0, 0, 0.4);
  color: white;
  z-index: 1;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.3s ease, visibility 0.3s ease;
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
}

.video-overlay-title h3 {
  margin: 0;
  white-space: nowrap;
  font-weight: bold;
  font-size: calc(1vw + 0.6em);
}

/* Progress overlay */
.video-progress-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 10px;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
  color: white;
  z-index: 1;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.3s ease, visibility 0.3s ease;
}

.overlay-visible {
  opacity: 1;
  visibility: visible;
}

/* Controls wrapper */
.controls-wrapper {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

/* Playback controls */
.playback-control {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.control-icon {
  width: 20px;
  height: 20px;
}

/* Subtitle toggle */
.subtitle-toggle {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  font-weight: bold;
  padding: 0 8px;
  font-size: 16px;
  user-select: none;
}

.subtitle-toggle.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Volume controls */
.volume-control {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.volume-icon {
  cursor: pointer;
  width: 20px;
  height: 20px;
}

.volume-bar {
  position: relative;
  height: 6px;
  width: 60px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  cursor: pointer;
}

.volume-bar-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: #42b883;
  border-radius: 2px;
}

/* Progress bar */
.progress-bar {
  position: relative;
  height: 20px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
  margin-bottom: 5px;
  cursor: pointer;
  user-select: none;
}

.progress-bar-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: #42b883;
  border-radius: 3px;
}

.time-display {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  color: white;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
  font-weight: bold;
  font-size: 14px;
}
</style>