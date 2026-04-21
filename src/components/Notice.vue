<script setup lang="ts">
import { ref, onUnmounted } from 'vue';

const message = ref('');
const isVisible = ref(false);
let timeoutId: any = null;

const show = (msg: string, duration: number) => {
  message.value = msg;
  isVisible.value = true;
  
  if (timeoutId) clearTimeout(timeoutId);
  timeoutId = setTimeout(() => {
    isVisible.value = false;
  }, duration);
};

defineExpose({ show });

onUnmounted(() => {
  if (timeoutId) clearTimeout(timeoutId);
});
</script>

<template>
  <div v-if="isVisible" class="notice-content" v-html="message"></div>
</template>

<style scoped>

.notice-content {
  position: fixed;
  top: 30px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.4);;
  border-radius: 8px;
  padding: 5px 15px 5px 15px;
  font-size: calc(1vw + 1em);
  color: white;
  z-index: 1000;
  font-weight: bold;;
}

</style>