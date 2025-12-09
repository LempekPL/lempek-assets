<script setup lang="ts">
const props = defineProps<{
  onSuccess?: () => void;
  current: string | null | undefined;
}>();
type FileTransfer = {
  name: string | null
  file: File
  error: string
  progress: number
  status: 'success' | 'none'
}
const { isFileDrag } = useDragEvents();

const isDragOver = computed(() => dragAmount.value > 0);
const dragAmount = ref(0);
const addFileBox = ref(false);
const loading = ref(false);
const files = reactive<FileTransfer[]>([]);

function onDrop(event: DragEvent) {
  if (!isFileDrag(event)) return;
  event.preventDefault();
  dragAmount.value = 0;
  const fil = event.dataTransfer?.files;
  if (fil && fil.length > 0) {
    for (const file of fil) {
      files.push({name: file.name, file: file, error: '', progress: 0, status: 'none'});
    }
  } else {
    files.length = 0;
  }
  addFileBox.value = true;
}

function onCancel() {
  if (loading.value) {
    activeXHR.value?.abort();
  }
  files.length = 0;
  addFileBox.value = false
}

const config = useRuntimeConfig()
const activeXHR = ref<XMLHttpRequest | null>(null);

async function onSubmit() {
  if (files.length == 0) {
    return
  }
  loading.value = true;
  for (const file of files) {
    if (file.status === 'success') {
      continue;
    }
    file.progress = 0;
    try {
      const formData = new FormData();
      formData.append('file', file.file);
      formData.append('name', file.name || file.file.name);
      if (props.current) formData.append('folder', props.current);

      await new Promise<void>((resolve, reject) => {
        const xhr = new XMLHttpRequest();
        activeXHR.value = xhr;
        xhr.open('POST', config.public.apiBase + '/upload');
        xhr.withCredentials = true;

        xhr.upload.onprogress = (event) => {
          if (event.lengthComputable) {
            file.progress = Math.round((event.loaded / event.total) * 100000) / 1000;
          }
        };

        xhr.onload = async () => {
          if (xhr.status >= 200 && xhr.status < 300) {
            file.progress = 100;
            file.status = 'success';
            resolve();
          } else {
            reject(JSON.parse(xhr.response));
          }
        };

        xhr.onerror = () => {
          reject(JSON.parse(xhr.responseText));
        };

        xhr.onabort = () => {
          file.progress = 0;
        }

        xhr.send(formData);
      });
    } catch (err: any) {
      file.error = err?.detail || 'Błąd wysyłania';
    } finally {
      activeXHR.value = null;
    }
  }
  if (!files.some((file) => file.error != '')) {
    addFileBox.value = false;
    props.onSuccess && props.onSuccess();
  }
  loading.value = false;
}
</script>

<template>
  <div
      @dragleave="(e) => {if (isFileDrag(e)) dragAmount--}"
      @dragenter="(e) => {if (isFileDrag(e)) dragAmount++}"
      @drop="onDrop"
      @dragover.prevent
      :class="{ 'drag-over': isDragOver }"
      class="drop-area"
  >
    <slot/>
    <div v-show="isDragOver" class="blackout">
      <div>
        <Icon class="upload-icon" name="material-symbols:upload-2-rounded"/>
      </div>
      <p>Prześlij plik</p>
    </div>

    <BoxModal
        :show="addFileBox"
        :loading="loading"
        :onSubmit="onSubmit"
        :onCancel="onCancel"
    >
      <div v-for="(file, num) in files" :key="num" style="width: 100%;">
        <div class="name-box">
          <PartInput :id="'name-'+(num+1)" :name="'Plik '+(num+1)"
                     :placeholder="file.file.name" v-model="file.name" style="width: 100%;"/>
          <PartProgress v-if="loading" class="progress-bar" :value="file.progress" :max="100"
                        :text="file.progress.toFixed(1)+'%'" bg-color="#aaa"
                        color="var(--accent-color)"/>
        </div>
        <BoxError v-if="file.error && !file.error.success" :message="file.error"/>
      </div>
      <template #cancel>
        <PartButton type="button" @click="onCancel"
                    :style="{ backgroundColor: loading ? 'var(--red-button-color)' : '' }">Anuluj
        </PartButton>
      </template>
      <template #action>
        <PartButton type="submit" :disabled="loading" style="background: var(--green-button-color)">
          Wyślij plik
        </PartButton>
      </template>
    </BoxModal>
  </div>
</template>

<style scoped>
.name-box {
  margin-top: .25rem;
  margin-bottom: .5rem;
  position: relative;

  .progress-bar {
    position: absolute;
    top: 0;
    bottom: 0;
    height: 100%;
    border-radius: 9999rem;
    background-color: var(--box-color);
  }
}

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