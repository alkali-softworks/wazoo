<script setup lang="ts">
import { onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const isElectron = window.electron !== undefined

const props = defineProps<{
  isOpen: boolean;
  onClose: () => void;
  wazooRef: any
}>();

const { t } = useI18n()

const handleMenuClick = (action: string) => {
  props.onClose()

  switch(action) {
    case 'search':
      props.wazooRef.openSearchModal()
      break;
    case 'add-player':
      props.wazooRef.addNewPlayer()
      break;
    case 'toggle-layout':
      props.wazooRef.toggleLayout()
      break;
    case 'toggle-files':
      props.wazooRef.toggleFiles()
      break;
    case 'settings':
      props.wazooRef.openSettingsModal()
      break;
    case 'help':
      props.wazooRef.openHelpModal()
      break;
    case 'quit':
      window.electron.invoke('close-app')
      break;
  }
}

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

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click="onClose">
    <div class="modal-content" @click.stop>
      <div class="modal-body">
          <div class="menu-item" @click="handleMenuClick('add-player')">{{ t('common.add_player') }}</div>
          <div class="menu-item" @click="handleMenuClick('toggle-layout')">{{ t('common.toggle_layout') }}</div>
          <div class="menu-item" @click="handleMenuClick('toggle-files')">{{ t('common.toggle_files') }}</div>
          <div class="menu-item" @click="handleMenuClick('search')">{{ t('common.search') }}</div>
          <div class="menu-item" @click="handleMenuClick('settings')">{{ t('common.settings') }}</div>
          <div class="menu-item" @click="handleMenuClick('help')">{{ t('common.help') }}</div>
          <div v-if="isElectron" class="menu-item" @click="handleMenuClick('quit')">{{ t('common.quit') }}</div>
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

.menu-item {
  padding: 8px 12px;
  cursor: pointer;
  color: #fff;
  transition: background-color 0.2s;
}

.menu-item:hover {
  background: #333;
}

.menu-item:last-child {
  border-radius: 0 0 6px 6px;
}


</style>