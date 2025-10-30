<script setup lang="ts">
const props = defineProps<{
  onSuccess?: () => void;
  current: string | null;
}>();

import { ref } from 'vue';

const isDragOver = ref(false);
const addFileBox = ref(false);
const file = ref<File | null>(null);

function onDragOver(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = true;
}

function onDragLeave(_event: DragEvent) {
  isDragOver.value = false;
}

function onDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = false;
  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    file.value = files.item(0);
    addFileBox.value = true;
  }
}

function handleSuccess() {
  addFileBox.value = false;
  props.onSuccess && props.onSuccess();
}

</script>

<template>
  <div
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
      :class="{ 'drag-over': isDragOver }"
      class="drop-area"
  >
    <slot />
    <div v-show="isDragOver" class="blackout">
      <div>
        <Icon class="upload-icon" name="material-symbols:upload-2-rounded"/>
      </div>
      <p>Prześlij plik</p>
    </div>

    <BoxFileUpload
        :show="addFileBox"
        @close="addFileBox = false"
        @success="handleSuccess"
        :parent-id="current || undefined"
        :file="file"
    />
  </div>
</template>

<style scoped>
.blackout {
  pointer-events: none;
  position: absolute;

  top: 1rem;
  left: 1rem;
  width: calc(100% - 2rem);
  height: calc(100% - 2rem);
  background-color: rgba(50, 50, 50, 0.5);
  border-radius: 1rem;
  border: 1px solid var(--accent-color);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  > div {
    padding: 2rem;
    border-radius: 1rem;
    width: 15vw;
    height: 15vw;

    .upload-icon {
      color: white;
      width: 100%;
      height: 100%;
    }
  }
  > p {
    font-size: 2rem;
  }
}
</style>