<script setup lang="ts">
const errorMessage = {success: false, detail: 'problem ze stworzeniem foldera'}

const props = defineProps({
  show: {
    type: Boolean,
    required: true
  },
});
const emit = defineEmits(['close']);
</script>

<template>
  <transition name="fade">
    <div class="bg" v-if="show" @click.self="emit('close')">
      <div class="big-box-menu">
        <PartInput id="name" name="Nazwa" style="width: 100%"/>
        <div class="bottom">
          <PartButton type="submit" style="background: #982727" @click.self="emit('close')">Anuluj</PartButton>
          <PartButton type="submit">Dodaj folder</PartButton>
        </div>
      </div>
      <div class="err">
        <p v-if="errorMessage && !errorMessage.success" class="err-text">{{ errorMessage.detail }}</p>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.err {
  position: absolute;
  left: 50%;
  top: calc(50% + 8rem);
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

  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  transition: 300ms;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.23s cubic-bezier(.42,.13,.4,1.13);
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
.fade-enter-to, .fade-leave-from {
  opacity: 1;
}
</style>