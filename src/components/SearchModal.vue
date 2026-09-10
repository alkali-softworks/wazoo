<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed } from 'vue';
import { Button } from '@/components/ui/button'
import {
  TagsInput,
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
  TagsInputItemText,
} from '@/components/ui/tags-input'
import { Search as SearchIcon } from 'lucide-vue-next'
import { log } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'

const { t } = useI18n();

const settingsStore = useSettingsStore();
const inputRef = ref<any>(null);
const segments = ref<string[]>([]);
const currentInputText = ref('');

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
  if (!query) {
    segments.value = [];
    currentInputText.value = '';
    return;
  }
  const newSegments = query.split(',').map(s => s.trim()).filter(Boolean);
  segments.value = Array.from(new Set(newSegments));
  currentInputText.value = '';
}

watch(segments, () => {
  currentInputText.value = '';
});

const onInputChange = (e: Event) => {
  currentInputText.value = (e.target as HTMLInputElement).value;
}

const handleKeyDown = (e: KeyboardEvent) => {
  e.stopPropagation();
  if (e.key === 'Escape') {
    e.preventDefault();
    props.onClose();
  } else if (e.key === 'Enter') {
    e.preventDefault();
    handleSearch();
  }
}

function handleSearch() {
  const comp = inputRef.value as any;
  const el = comp?.$el ?? comp;
  const inputEl = el instanceof HTMLInputElement ? el : el?.querySelector?.('input');
  const inputVal = inputEl ? inputEl.value : currentInputText.value;
  const pending = (inputVal || currentInputText.value || '').trim();
  if (pending && !segments.value.includes(pending)) {
    segments.value.push(pending);
  }
  if (inputEl) {
    inputEl.value = '';
  }
  currentInputText.value = '';
  
  const fullQuery = segments.value.join(', ');
  log('fullQuery:', fullQuery);
  
  // Pass the folder value into the emit
  const folderToEmit = selectedFolders.value.length === 0 ? 'All' : selectedFolders.value;
  emit('search', fullQuery, folderToEmit);
  
  props.onClose();
}

function removeSegment(segment: string) {
  const index = segments.value.indexOf(segment);
  if (index !== -1) {
    segments.value.splice(index, 1);
  }
}

const focusInput = () => {
  const comp = inputRef.value as any;
  if (comp?.focus) {
    comp.focus();
  } else {
    const el = comp?.$el ?? comp;
    if (el instanceof HTMLInputElement) {
      el.focus();
    } else if (el?.querySelector) {
      el.querySelector('input')?.focus();
    }
  }
}

defineExpose({
  setQuery,
  setFolder
});

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    window.addEventListener('keydown', handleKeyDown);
    await nextTick();
    focusInput();
  } else {
    window.removeEventListener('keydown', handleKeyDown);
  }
}, { immediate: true });

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

        <div class="flex gap-2 items-center">
          <TagsInput
            v-model="segments"
            :add-on-paste="true"
            class="flex-1 min-h-10 bg-[#2a2a2a] border border-[#3f3f3f] rounded-md px-3 py-1.5 focus-within:border-[#4f4f4f] focus-within:ring-2 focus-within:ring-[rgba(255,255,255,0.1)] transition-colors"
          >
            <TagsInputItem
              v-for="item in segments"
              :key="item"
              :value="item"
              class="bg-[#3a3a3a] border border-[#4f4f4f] text-white hover:bg-[#444] transition-colors rounded px-2 py-0.5 text-sm"
            >
              <TagsInputItemText class="text-sm" />
              <TagsInputItemDelete class="text-zinc-400 hover:text-white" />
            </TagsInputItem>
            <TagsInputInput
              ref="inputRef"
              :placeholder="segments.length === 0 ? t('search.placeholder') : ''"
              class="text-base min-h-7 placeholder:text-zinc-500 text-white"
              @input="onInputChange"
              @keydown.enter="handleKeyDown"
            />
          </TagsInput>
          <Button @click="handleSearch"><SearchIcon class="w-4 h-4" /></Button>
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
</style>