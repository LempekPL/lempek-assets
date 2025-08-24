<script setup lang="ts">
defineProps<{
  message: string;
}>()
</script>

<template>
  <div class="ok-box">
    <div>
      <Icon name="material-symbols:check-rounded"/>
    </div>
    <p>{{ message }}</p>
  </div>
</template>

<style scoped>
.ok-box {
  display: flex;
  flex-direction: row;
  border: 2px solid var(--green-button-color);
  align-items: center;
  border-radius: .5rem;

  div {
    height: 100%;
    background-color: var(--green-button-color);
    span {
      font-size: 2rem;
    }
  }

  p {
    padding: .5rem;
  }
}
</style>