<script setup lang="ts">
import { ref, shallowRef, watch, onUnmounted, nextTick } from 'vue'
import ASS from 'assjs'
import { normalizeSubtitles } from '@/lib/subtitles'

const props = withDefaults(defineProps<{
  subtitleContent: string
  videoElement?: HTMLVideoElement | null
  enabled?: boolean
  currentTime?: number
}>(), {
  videoElement: null,
  enabled: true,
  currentTime: 0
})

const containerRef = ref<HTMLDivElement | null>(null)
const assInstance = shallowRef<ASS | null>(null)


const destroyASS = () => {
  if (assInstance.value) {
    try {
      assInstance.value.destroy()
    } catch (e) {
      console.error('Error destroying ASS instance:', e)
    }
    assInstance.value = null
  }
}

const initASS = () => {
  destroyASS()

  const content = props.subtitleContent
  const video = props.videoElement
  const container = containerRef.value

  if (!content || !video || !container) return

  const normalized = normalizeSubtitles(content)
  if (!normalized) return

  try {
    const ass = new ASS(normalized, video, {
      container,
      resampling: 'video_height'
    })

    if (!props.enabled) {
      ass.hide()
    }

    assInstance.value = ass
  } catch (err) {
    console.error('Failed to initialize ASS subtitle renderer:', err)
  }
}

// Watch subtitle content and video element availability
watch([() => props.subtitleContent, () => props.videoElement], () => {
  nextTick(() => {
    initASS()
  })
}, { immediate: true })

// Watch subtitle visibility toggle
watch(() => props.enabled, (isEnabled) => {
  if (assInstance.value) {
    if (isEnabled) {
      assInstance.value.show()
    } else {
      assInstance.value.hide()
    }
  }
})

onUnmounted(() => {
  destroyASS()
})

defineExpose({
  getASS: () => assInstance.value,
  reload: initASS
})
</script>

<template>
  <div 
    ref="containerRef" 
    class="subtitle-overlay"
    :class="{ 'hidden': !props.enabled }"
  />
</template>

<style scoped>
.subtitle-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  overflow: hidden;
  z-index: 2;
}

.subtitle-overlay.hidden {
  display: none;
}
</style>