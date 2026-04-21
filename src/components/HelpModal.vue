<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  isOpen: boolean;
  onClose: () => void;
}>();

const { t } = useI18n()

const handleKeyDown = (e: KeyboardEvent) => {
  e.preventDefault();
  e.stopPropagation();
  if (e.key === 'Escape' || e.key === '?') {
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

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});


const shortcuts = computed(() => [
  { key: '?', description: t('help.shortcuts.toggle_help') },
  { key: '5', description: t('help.shortcuts.toggle_scroll') },
  { key: 'h', description: t('help.shortcuts.toggle_file_picker') },
  { key: 'n', description: t('help.shortcuts.add_player') },
  { key: 'x', description: t('help.shortcuts.remove_player') },
  { key: 'Tab', description: t('help.shortcuts.focus_next') },
  { key: 'l', description: t('help.shortcuts.toggle_layout') },
  { key: 'c', description: t('help.shortcuts.toggle_subtitles') },
  { key: '< OR >', description: t('help.shortcuts.prev_next_frame') },
  { key: '[space]', description: t('help.shortcuts.play_pause') },
  { key: '↓ ↑', description: t('help.shortcuts.prev_next_video') },
  { key: '← →', description: t('help.shortcuts.seek_back_forward') },
  { key: 's', description: t('help.shortcuts.toggle_play_mode') },
  { key: 'm', description: t('help.shortcuts.toggle_mute') },
  { key: '[ OR ]', description: t('help.shortcuts.adjust_volume') },
  { key: 'j OR /', description: t('help.shortcuts.search_videos') },
  { key: 'f OR F11', description: t('help.shortcuts.fullscreen') },
  { key: 'Alt + X', description: t('help.shortcuts.close_app') },
  { key: 'Alt + Drag', description: t('help.shortcuts.move_window') },
  { key: 'd', description: t('help.shortcuts.dad_joke') },
]);
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click="onClose">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>{{ t('help.title') }}</h2>
        <button class="close-modal" @click="onClose">✕</button>
      </div>
      <div class="shortcuts-list">
        <div v-for="(shortcut, index) in shortcuts" :key="index" class="shortcut-item">
          <span class="shortcut-key">{{ shortcut.key }}</span>
          <span class="shortcut-description">{{ shortcut.description }}</span>
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

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.shortcut-key {
  background: #333;
  padding: 4px 8px;
  border-radius: 4px;
  font-family: monospace;
  font-weight: bold;
}

.shortcut-description {
  color: #ddd;
}
</style>