<script setup lang="ts">
defineProps<{
  value: number;
  max: number;
  text?: string;
  bgColor?: string;
  color?: string;
}>()
</script>

<template>
  <div class="progress-bar-element" :style="{ backgroundColor: bgColor || '#e0e0e0' }">
    <div class="actual-progress" :style="{
      width: value / max * 100 + '%',
      backgroundColor: color || '#82d226'
    }" />
    <span class="progress-text" v-show="text">{{ text }}</span>
  </div>
</template>

<style scoped>
.progress-bar-element {
  display: inline-block;
  block-size: 1rem;
  width: 100%;
  position: relative;
  overflow: hidden;

  .actual-progress {
    height: 100%;
    animation: shimmer 1.7s infinite linear;
  }

  .progress-text {
    position: absolute;
    top: 0;
    left: 50%;
    font-size: 1rem;
    transform: translateX(-50%);
  }
}

@keyframes shimmer {
  0%   { filter: brightness(1); }
  50%  { filter: brightness(1.16); }
  100% { filter: brightness(1); }
}
</style>