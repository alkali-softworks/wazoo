<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed } from 'vue';
import { Button } from '@/components/ui/button'
import { Search as SearchIcon } from 'lucide-vue-next'
import { log } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'

const { t } = useI18n();

const settingsStore = useSettingsStore();
const inputRef = ref<HTMLInputElement | null>(null)
const segments = ref<Set<string>>(new Set());
const selectedSegmentIndex = ref(-1); 
const segmentElements = ref<Map<number, HTMLElement>>(new Map());

const props = defineProps<{
  isOpen: boolean;
  onClose: () => void;
  folderPref?: string;
}>();

const selectedFolders = ref<string[]>(
  props.folderPref && props.folderPref !== '' && props.folderPref !== 'All' 
    ? (Array.isArray(props.folderPref) ? props.folderPref : [props.folderPref]) 
    : []
);
const searchQuery = ref('')

const isAllSelected = computed(() => selectedFolders.value.length === 0);

const toggleAll = () => {
  selectedFolders.value = [];
}

const emit = defineEmits(['search'])

const folderOptions = computed(() => {
  const options = [{ value: 'All', label: t('common.all') }]
  
  settingsStore.mediaFolders.forEach((folder) => {
    const lastSegment = folder.split(/[/\\]/).filter(Boolean).pop() || folder
    const label = lastSegment.replace(/\b\w/g, (c) => c.toUpperCase())
    options.push({ value: folder, label })
  })

  return options
})

const setFolder = (folder: string | string[]) => {
  log(`setFolder: "${folder}"`);
  if (Array.isArray(folder)) {
    selectedFolders.value = [...folder];
  } else if (folder === 'All' || folder === '') {
    selectedFolders.value = [];
  } else {
    selectedFolders.value = [folder];
  }
}

const setQuery = (query: string) => {
  const newSegments = query.split(',').map(s => s.trim()).filter(Boolean);
  segments.value = new Set(newSegments);
  searchQuery.value = '';
}

const handleKeyDown = (e: KeyboardEvent) => {
  e.stopPropagation();
  if (e.key === 'Escape') {
    e.preventDefault();
    props.onClose();
  } else if (e.key === ',' && searchQuery.value.trim()) {
    segments.value.add(searchQuery.value.trim());
    searchQuery.value = '';
    e.preventDefault();
  } else if (e.key === 'Enter') {
    handleSearch();
  } else if (e.key === 'ArrowDown' && selectedSegmentIndex.value === -1) {
    if (segments.value.size > 0) {
      selectedSegmentIndex.value = 0;
      e.preventDefault();
      nextTick(() => {
        segmentElements.value.get(0)?.focus();
      });
    }
  }
}

const handleSegmentKeyDown = (e: KeyboardEvent, segment: string, index: number) => {
  if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault();
    removeSegment(segment);
    if (index > 0) {
      selectedSegmentIndex.value = index - 1;
      segmentElements.value.get(index - 1)?.focus();
    } else {
      selectedSegmentIndex.value = -1;
      inputRef.value?.focus();
    }

  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
    e.preventDefault();
    if (index > 0) {
      selectedSegmentIndex.value = index - 1;
      segmentElements.value.get(index - 1)?.focus();
    } else {
      selectedSegmentIndex.value = -1;
      inputRef.value?.focus();
    }

  } else if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
    e.preventDefault();
    if (index < segments.value.size - 1) {
      selectedSegmentIndex.value = index + 1;
      segmentElements.value.get(index + 1)?.focus();
    } else {
      selectedSegmentIndex.value = -1;
      inputRef.value?.focus();
    }
  } else if (e.key === 'Enter') {
    handleSearch();
  }
}

function handleSearch() {
  if (searchQuery.value.trim() !== '') {
    segments.value.add(searchQuery.value.trim());
  }
  searchQuery.value = '';
  
  const fullQuery = Array.from(segments.value).join(', ');
  log('fullQuery:', fullQuery);
  
  // Pass the folder value into the emit
  const folderToEmit = selectedFolders.value.length === 0 ? 'All' : selectedFolders.value;
  emit('search', fullQuery, folderToEmit);
  
  props.onClose();
}

function removeSegment(segment: string) {
  segments.value.delete(segment);
}
function handleBlur(index:number) {
  if (selectedSegmentIndex.value === index)
  selectedSegmentIndex.value = -1
}

defineExpose({
  setQuery,
  setFolder
});

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    window.addEventListener('keydown', handleKeyDown);
    await nextTick();
    inputRef.value?.focus();
    selectedSegmentIndex.value = -1;
    segmentElements.value.clear();
  } else {
    window.removeEventListener('keydown', handleKeyDown);
    segmentElements.value.clear();
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click="onClose">
    <div class="modal-content" @click.stop>
      <div class="grid gap-4">
        
        <div class="flex flex-col gap-2">
          <p class="text-sm font-medium">{{ t('search.folder') }}</p>
          <div class="folders-list">
            <label class="folder-label" :class="{ 'active': isAllSelected }">
              <input type="checkbox" :checked="isAllSelected" @change="toggleAll" class="folder-checkbox" />
              <span>{{ t('common.all') }}</span>
            </label>
            <label v-for="option in folderOptions.slice(1)" 
                   :key="option.value" 
                   class="folder-label"
                   :class="{ 'active': selectedFolders.includes(option.value) }">
              <input type="checkbox" 
                     v-model="selectedFolders" 
                     :value="option.value"
                     class="folder-checkbox" />
              <span>{{ option.label }}</span>
            </label>
          </div>
        </div>

        <div class="flex gap-2">
          <input
            ref="inputRef"
            v-model="searchQuery"
            type="text"
            :placeholder="t('search.placeholder')"
            class="flex-1 h-10 px-3 py-2 rounded-md border border-input bg-background text-lg ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
            @keydown="handleKeyDown"
          />
          <Button @click="handleSearch"><SearchIcon class="w-4 h-4" /></Button>
        </div>
        
        <div v-if="segments.size" class="segments-container">
          <div v-for="(segment, index) in segments" 
            :key="segment" 
            class="segment"
            :class="{ 'segment-selected': index === selectedSegmentIndex }"
            tabindex="0"
            :ref="(el) => { if (el) segmentElements.set(index, el as HTMLElement) }"
            @keydown="(e) => handleSegmentKeyDown(e, segment, index)"
            @focus="selectedSegmentIndex = index"
            @blur="handleBlur(index)"
            @click="selectedSegmentIndex = index">
            {{ segment }}
            <button class="remove-segment" 
              @click.stop="removeSegment(segment)">×</button>
          </div>
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
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100000;
  backdrop-filter: blur(20px);
}

.modal-content {
  background: #1f1f1f;
  border-radius: 8px;
  padding: 20px;
  min-width: 300px;
  max-width: 500px;
  width: 100%; /* Ensure it takes width on smaller screens */
  color: white;
  box-shadow: 0 10px 25px rgba(0,0,0,0.5);
}

.folders-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  max-height: 120px;
  overflow-y: auto;
  padding: 4px;
  background: #1a1a1a;
  border-radius: 6px;
  border: 1px solid #333;
}

.folder-label {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  background: #2a2a2a;
  border: 1px solid #3f3f3f;
  border-radius: 20px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
  user-select: none;
}

.folder-label:hover {
  background: #333;
}

.folder-label.active {
  background: #3b82f6;
  border-color: #60a5fa;
  color: white;
}

.folder-checkbox {
  display: none;
}

input {
  background: #2a2a2a;
  color: white;
  border: 1px solid #3f3f3f;
}

input:focus {
  border-color: #4f4f4f;
  outline: none;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.1);
}

input::placeholder {
  color: #666;
}

.segments-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.segment {
  background: #2a2a2a;
  border: 1px solid #3f3f3f;
  border-radius: 4px;
  padding: 4px 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  outline: none;
}

.segment-selected {
  background: #3a3a3a;
  border-color: #5f5f5f;
}

.remove-segment {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 0 2px;
  font-size: 16px;
  line-height: 1;
}

.remove-segment:hover {
  color: #fff;
}
</style>