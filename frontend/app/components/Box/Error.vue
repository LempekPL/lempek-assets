<script setup lang="ts">
defineProps<{
  message: string;
}>()
</script>

<template>
  <div class="err-box">
    <div>
      <Icon name="material-symbols:exclamation-rounded"/>
    </div>
    <p>{{ message }}</p>
  </div>
</template>

<style scoped>
.err-box {
  display: flex;
  flex-direction: row;
  border: 2px solid var(--red-button-color);
  align-items: center;
  border-radius: .5rem;

  div {
    height: 100%;
    background-color: var(--red-button-color);
    span {
      font-size: 2rem;
    }
  }

  p {
    padding: .5rem;
  }
}
</style>