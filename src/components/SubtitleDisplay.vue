<script setup lang="ts">
import { ref, watch } from 'vue'

interface Subtitle {
  start: number
  end: number
  text: string
}

const props = defineProps({
  subtitleContent: {
    type: String,
    required: true
  },
  currentTime: {
    type: Number,
    required: true
  }
})

const parsedSubtitles = ref<Subtitle[]>([])
const currentSubtitle = ref<Subtitle | null>(null)
const lastSubtitleIndex = ref<number>(0)

const timeToSeconds = (timeString: string) => {

  if (typeof timeString !== 'string' || !timeString.length) return 0

  // Handle ASS format (h:mm:ss.cc)
  if (timeString.includes('.')) {
    const [time, centiseconds] = timeString.split('.')
    const [hours, minutes, seconds] = time.split(':')
    return (
      parseInt(hours) * 3600 +
      parseInt(minutes) * 60 +
      parseInt(seconds) +
      parseInt(centiseconds) / 100
    )
  }
  // Handle SRT format (HH:MM:SS,mmm)
  const [hours, minutes, seconds] = timeString.split(':')

  if (typeof seconds !== 'string' || !seconds.length) return 0
  const [secs, ms] = seconds.split(',')

  return (
    parseInt(hours) * 3600 +
    parseInt(minutes) * 60 +
    parseInt(secs) +
    parseInt(ms) / 1000
  )
}

const parseSubtitles = (content: string): Subtitle[] => {
  // Check if content is ASS/SSA format
  if (content.includes('[Script Info]')) {
    return parseASSSubtitles(content)
  }
  // Otherwise assume SRT format
  return parseSRTSubtitles(content)
}

const parseASSSubtitles = (content: string): Subtitle[] => {
  const subtitles = []
  const lines = content.split('\n')
  let isEvents = false
  let formatLine = null

  for (const line of lines) {
    if (line.startsWith('[Events]')) {
      isEvents = true
      continue
    }

    if (isEvents) {
      if (line.startsWith('Format:')) {
        formatLine = line.substring(8).split(',').map(s => s.trim())
        continue
      }

      if (line.startsWith('Dialogue:')) {
        const parts = line.substring(9).split(',')
        const startIdx = formatLine.indexOf('Start')
        const endIdx = formatLine.indexOf('End')
        const textIdx = formatLine.indexOf('Text')

        if (startIdx !== -1 && endIdx !== -1 && textIdx !== -1) {
          const start = timeToSeconds(parts[startIdx])
          const end = timeToSeconds(parts[endIdx])
          // Join remaining parts as text and remove style codes
          const text = parts.slice(textIdx).join(',')
            .replace(/{[^}]*}/g, '') // Remove style codes
            .replace(/\\N/g, '<br>')
            .replace(/\\n/g, '<br>')
            .trim()

          subtitles.push({ start, end, text })
        }
      }
    }
  }

  return subtitles
}

const parseSRTSubtitles = (content: string): Subtitle[] => {
  const subtitles: Subtitle[] = []
  const blocks = content.trim().split('\n\n')

  blocks.forEach(block => {
    const lines = block.split('\n')
    if (lines.length >= 3) {
      const timeLine = lines[1]
      const [startTime, endTime] = timeLine.split(' --> ').map(timeToSeconds)
      const text = lines.slice(2).join('\n')

      subtitles.push({
        start: startTime,
        end: endTime,
        text: text
      })
    }
  })

  return subtitles
}

const updateCurrentSubtitle = (currentTime: number): void => {
  const subtitles = parsedSubtitles.value

  // Early return if no subtitles
  if (!subtitles.length) {
    currentSubtitle.value = null
    return
  }

  // First check if we're still in the same subtitle
  const lastSub = subtitles[lastSubtitleIndex.value]
  if (lastSub && currentTime >= lastSub.start && currentTime <= lastSub.end) {
    currentSubtitle.value = lastSub
    return
  }

  // Then check the next subtitle (most common case)
  const nextIndex = lastSubtitleIndex.value + 1
  if (nextIndex < subtitles.length) {
    const nextSub = subtitles[nextIndex]
    if (currentTime >= nextSub.start && currentTime <= nextSub.end) {
      lastSubtitleIndex.value = nextIndex
      currentSubtitle.value = nextSub
      return
    }
  }

  // If not found, do binary search
  let left = 0
  let right = subtitles.length - 1

  while (left <= right) {
    const mid = Math.floor((left + right) / 2)
    const sub = subtitles[mid]

    if (currentTime >= sub.start && currentTime <= sub.end) {
      lastSubtitleIndex.value = mid
      currentSubtitle.value = sub
      return
    }

    if (currentTime < sub.start) {
      right = mid - 1
    } else {
      left = mid + 1
    }
  }

  // No subtitle found for current time
  currentSubtitle.value = null
  
  // Update lastSubtitleIndex to the closest subtitle before current time
  // This helps optimize the next search
  lastSubtitleIndex.value = Math.max(0, left - 1)
}

watch(() => props.subtitleContent, (newContent) => {
  if (newContent) {
    parsedSubtitles.value = parseSubtitles(newContent)
  } else {
    parsedSubtitles.value = []
  }
}, { immediate: true })

watch(() => props.currentTime, (newTime) => {
  updateCurrentSubtitle(newTime)
}, { immediate: true })
</script>

<template>
  <div class="subtitles">
    <div v-if="currentSubtitle" 
      class="subtitle-text" 
      v-html="currentSubtitle.text">
    </div>
  </div>
</template>

<style scoped>
.subtitles {
  position: absolute;
  bottom: 60px;
  left: 0;
  right: 0;
  text-align: center;
  z-index: 2;
}

.subtitle-text {
  display: inline-block;
  text-shadow: 2px 2px 4px black;
  color: white;
  font-size: calc(1vw + 1em);
  font-weight: bold;
  max-width: 80%;
  margin: 0 auto;
}
</style>