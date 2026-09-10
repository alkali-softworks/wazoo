<script setup lang="ts">
import { cn } from '@/lib/utils'
import { TagsInputInput, type TagsInputInputProps, useForwardProps } from 'reka-ui'
import { computed, ref, type HTMLAttributes } from 'vue'

const props = defineProps<TagsInputInputProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
const inputEl = ref<any>(null)

defineExpose({
  $el: computed(() => inputEl.value?.$el ?? inputEl.value),
  focus: () => {
    const el = inputEl.value?.$el ?? inputEl.value
    if (el instanceof HTMLInputElement) {
      el.focus()
    } else if (el?.querySelector) {
      el.querySelector('input')?.focus()
    }
  }
})
</script>

<template>
  <TagsInputInput
    ref="inputEl"
    v-bind="forwardedProps"
    :class="cn('text-sm min-h-6 focus:outline-none flex-1 bg-transparent px-1 placeholder:text-muted-foreground', props.class)"
  />
</template>
