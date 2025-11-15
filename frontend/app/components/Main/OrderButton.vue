<script setup lang="ts">
defineProps<{
  selected?: boolean,
}>();
</script>

<template>
  <button>
    <Icon v-if="selected" name="material-symbols:radio-button-checked-outline" class="icon"/>
    <Icon v-else name="material-symbols:radio-button-unchecked" class="icon"/>
    <span><slot/></span>
  </button>
</template>

<style lang="scss" scoped>
button {
  background: var(--box-color);
  padding: 1rem;
  cursor: pointer;
  border: none;
  font-size: 1rem;
  color: var(--text-color);
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;

  span {
    text-align: center;
    width: 100%;
  }

  .icon {
    width: 1.5rem;
    transform: scale(1.25);
  }

  &:first-child {
    border-radius: 1rem 1rem 0 0;
  }

  &:last-child {
    border-radius: 0 0 1rem 1rem;
  }

  &:hover {
    filter: brightness(75%);
  }

  &:active {
    filter: brightness(50%);
  }
}
</style>