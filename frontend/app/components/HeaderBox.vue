<script setup lang="ts">
import type {CSSProperties} from "vue";

defineProps<{
  width: CSSProperties['width'],
}>()
</script>

<template>
  <div class="box">
    <p><slot name="header"/></p>
    <div>
      <slot/>
    </div>
  </div>
</template>

<style scoped>
.box {
  padding: 1rem;
  width: v-bind(width);

  p {
    color: var(--text-color);
    font-weight: bold;
    font-size: 1.5rem;
    padding: 0.5rem;
  }

  > div {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    background-color: var(--box-color);
    border-radius: 1rem;
    padding: 1rem;
  }
}
</style>