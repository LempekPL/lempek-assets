<script setup lang="ts">
defineProps<{
  show: boolean
  loading: boolean
  errorMessage?: string
  onSubmit: () => void
  onCancel: () => void
}>();
</script>

<template>
  <transition name="fade">
    <div class="bg" v-show="show" @click.self="onCancel">
      <form class="big-box-menu" @submit.prevent="onSubmit">
        <slot/>
        <div class="bottom">
          <PartButton type="button" @click="onCancel" :disabled="loading">Anuluj</PartButton>
          <slot name="action">
            <PartButton type="submit" :disabled="loading">OK</PartButton>
          </slot>
        </div>
      </form>
      <div class="err" v-if="errorMessage">
        <p class="err-text">{{ errorMessage }}</p>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.err {
  position: fixed;
  left: 50%;
  top: calc(50% + 12rem);
  transform: translate(-50%, -50%);
}

.err-text {
  border: 2px solid white;
  background: #930000;
  padding: .5rem;
  border-radius: 9999rem;
  color: white;
}

.big-box-menu {
  background-color: var(--box-color);
  box-shadow: #000 .125rem .125rem 1rem .25rem;
  width: 30%;
  min-width: 20rem;
  padding: 2rem;
  border-radius: 2rem;
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-self: center;
  align-items: center;
  gap: 1rem;

  .bottom {
    display: flex;
    flex-direction: row;
    gap: 1rem;
  }
}

.bg {
  width: 100%;
  height: 100%;
  backdrop-filter: blur(10px);

  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  transition: 300ms;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.23s cubic-bezier(.42, .13, .4, 1.13);
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.fade-enter-to, .fade-leave-from {
  opacity: 1;
}
</style>
