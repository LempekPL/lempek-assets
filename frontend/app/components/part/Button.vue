<script setup lang="ts">
const buttonRef = ref<HTMLButtonElement | null>(null);
function contains(target: Node): boolean {
  return buttonRef.value?.contains(target) ?? false;
}
defineExpose({button: buttonRef, contains});
</script>

<template>
  <button ref="buttonRef">
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