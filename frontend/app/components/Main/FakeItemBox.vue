<template>
  <div class="item">
    <div class="icon" />
    <div class="texts">
      <div class="name-text"></div>
      <div class="author-text"></div>
    </div>
  </div>
</template>

<style scoped>
.item {
  user-select: none;
  padding: 2rem;
  height: 13rem;
  border-radius: 1rem;
  background-color: var(--box-color);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  text-align: center;
  cursor: pointer;

  .icon {
    padding: 3rem;
    background-color: #aaa;
    animation: loading-shimmer 1.5s infinite;
    border-radius: 1rem;
  }

  .texts {
    display: flex;
    flex-direction: column;
    width: 100%;
    align-items: center;
    gap: 4px;

    div {
      background-color: #aaa;
      animation: loading-shimmer 1.5s infinite;
      border-radius: 1rem;
    }

    .name-text {
      height: 1rem;
      width: 6rem;
    }

    .author-text {
      width: 4rem;
      height: .75rem;
    }
  }
}
</style>
<script setup lang="ts">
</script>