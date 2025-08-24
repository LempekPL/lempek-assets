<script setup lang="ts">
import { defineExpose, ref } from "vue";

const button = ref<HTMLButtonElement | null>(null);
defineExpose({ button });
</script>

<template>
  <button ref="button">
    <slot/>
  </button>
</template>

<style scoped>
button {
  color: var(--text-color);
  background: var(--button-color);
  border-radius: 99999rem;
  padding: 1rem;
  cursor: pointer;
  border: none;
  font-size: 1rem;
  transition: 300ms;

  &:hover {
    filter: brightness(75%);
  }

  &:disabled {
    cursor: not-allowed;
    filter: brightness(50%) grayscale(25%);
  }
}
</style>