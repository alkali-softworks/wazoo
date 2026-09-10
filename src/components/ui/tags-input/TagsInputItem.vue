<script setup lang="ts">
import { cn } from '@/lib/utils'
import { TagsInputItem, type TagsInputItemProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<TagsInputItemProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TagsInputItem
    v-bind="forwardedProps"
    :class="cn('flex h-6 items-center rounded-md bg-secondary text-secondary-foreground px-2 py-0.5 text-xs data-[state=active]:ring-2 data-[state=active]:ring-ring data-[state=active]:ring-offset-2 ring-offset-background gap-1', props.class)"
  >
    <slot />
  </TagsInputItem>
</template>
