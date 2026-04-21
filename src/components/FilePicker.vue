<script setup lang="ts">
import { ref, computed } from 'vue'
import { formatVideoTitle, cleanName } from '@/lib/utils'
import { usePlayerStore } from '@/stores/player'
import { useI18n } from 'vue-i18n'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Input } from '@/components/ui/input'

const { t } = useI18n()
const playerStore = usePlayerStore()
const files = computed(() => playerStore.videos)
const searchTerm = ref('')

// Transform files into folder structure with filtering
const folderStructure = computed(() => {
  const structure: Record<string, string[]> = {}
  const search = searchTerm.value.toLowerCase()
  
  files.value.forEach(file => {
    // Skip if file doesn't match search
    const title = formatVideoTitle(file).toLowerCase()
    if (search && !title.includes(search)) {
      return
    }

    const segments = file.replace('file://', '').split(/[\\\/]/).filter(Boolean)

    let folderName = segments[segments.length - 2] || 'Other'
    const genericPatterns = [
      /^Season\s+\d+/i,
      /^OVA\s+\d+/i,
      /^Specials?$/i,
      /^Disc\s+\d+/i,
      /^Vol(ume)?\s+\d+/i,
      /^\d+$/
    ]

    let folderPath = ''
    if (genericPatterns.some(p => p.test(folderName)) && segments.length >= 3) {
      folderPath = `${cleanName(segments[segments.length - 3])}: ${cleanName(folderName)}`
    } else {
      folderPath = cleanName(folderName)
    }

    if (!structure[folderPath]) {
      structure[folderPath] = []
    }
    structure[folderPath].push(file)
  })
  
  // Remove empty folders
  return Object.fromEntries(
    Object.entries(structure).filter(([_, files]) => files.length > 0)
  )
})

function playFile(file: string) {
  const playerRef = playerStore.currentPlayer
  playerRef.src = file
}
</script>

<template>
  <div class="file-picker">
    <div class="search-container">
      <Input
        v-model="searchTerm"
        :placeholder="t('file_picker.search_placeholder')"
        class="search-input"
        @keydown.stop
      />
    </div>
    <div class="folders-container">
      <div v-for="(files, folder) in folderStructure" :key="folder" class="folder-group">
        <Collapsible>
          <CollapsibleTrigger class="folder-trigger">
            <span class="folder-name">{{ folder }}</span>
            <span class="file-count">({{ files.length }})</span>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <ul>
              <li v-for="file in files" :key="file" @click="playFile(file)">
                {{ formatVideoTitle(file) }}
              </li>
            </ul>
          </CollapsibleContent>
        </Collapsible>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-picker {
  width: 100%;
  background-color: #333;
  padding: 16px;
  color: white;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.search-container {
  margin-bottom: 16px;
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  background-color: #444;
  border-color: #555;
  color: white;
}

.search-input::placeholder {
  color: #888;
}

.folders-container {
  flex-grow: 1;
  overflow-y: auto;
  max-height: calc(100vh - 200px); /* Adjust this value based on your layout */
  
  /* Scrollbar styling */
  scrollbar-width: thin;
  scrollbar-color: #666 #333;
}

.folders-container::-webkit-scrollbar {
  width: 8px;
}

.folders-container::-webkit-scrollbar-track {
  background: #333;
}

.folders-container::-webkit-scrollbar-thumb {
  background-color: #666;
  border-radius: 4px;
  border: 2px solid #333;
}

.folders-container::-webkit-scrollbar-thumb:hover {
  background-color: #888;
}

.folder-group {
  margin-bottom: 8px;
}

.folder-trigger {
  width: 100%;
  padding: 8px 12px;
  background: #444;
  border: none;
  color: white;
  text-align: left;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.folder-name {
  font-weight: 500;
}

.file-count {
  font-size: 0.9em;
  color: #888;
}

ul {
  list-style-type: none;
  padding: 8px 0 8px 24px;
  margin: 0;
}

li {
  padding: 8px 12px;
  cursor: pointer;
  color: #fff;
  transition: background-color 0.2s;
}

li:hover {
  background: #1f1f1f;
}
</style>