<script setup lang="ts">
const props = defineProps<{
  show: boolean
  parentId?: string | null
  file?: File | null
}>()
const emit = defineEmits(['close', 'success'])

const loading = ref(false)
const errorMessage = ref('')
const file = ref<File | null>(null);
const fileName = ref('');
const dragOver = ref(false);

watch(() => props.file, (newFile) => {
  if (!file.value) {
    file.value = newFile || null;
    fileName.value = fileName.value || newFile?.name || "";
  }
});

const config = useRuntimeConfig()
const progress = ref(0);
const activeXHR = ref<XMLHttpRequest | null>(null);

async function onSubmit() {
  if (!file.value) {
    errorMessage.value = 'Plik nie został wybrany.'
    return
  }
  loading.value = true;
  errorMessage.value = '';
  progress.value = 0;

  try {
    const formData = new FormData();
    formData.append('file', file.value);
    formData.append('name', fileName.value);
    if (props.parentId) formData.append('folder', props.parentId);

    await new Promise<void>((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      activeXHR.value = xhr;
      xhr.open('POST', config.public.apiBase + '/upload');
      xhr.withCredentials = true;

      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) {
          progress.value = Math.round((event.loaded / event.total) * 100000) / 1000;
        }
      };

      xhr.onload = async () => {
        loading.value = false;
        if (xhr.status >= 200 && xhr.status < 300) {
          progress.value = 100;
          fileName.value = '';
          file.value = null;
          emit('success');
          resolve();
        } else {
          reject(JSON.parse(xhr.response));
        }
      };

      xhr.onerror = () => {
        loading.value = false;
        reject(JSON.parse(xhr.responseText));
      };

      xhr.onabort = () => {
        loading.value = false;
        progress.value = 0;
      }

      xhr.send(formData);
    });
  } catch (err: any) {
    errorMessage.value = err?.detail || 'Błąd';
  } finally {
    loading.value = false;
    activeXHR.value = null;
  }
}

function onCancel() {
  if (loading.value) {
    activeXHR.value?.abort();
    return;
  }
  errorMessage.value = ''
  fileName.value = '';
  file.value = null;
  emit('close')
}


function changeFile(event: Event) {
  const target = event.target as HTMLInputElement;
  changeFileData(target.files);
}

function handleDrop(ev: DragEvent) {
  ev.preventDefault();
  dragOver.value = false;
  changeFileData(ev.dataTransfer?.files ?? null);
}

function changeFileData(fl: FileList | null) {
  if (fl && fl.length > 0) {
    file.value = fl.item(0);
    if (file.value) {
      fileName.value = file.value.name;
    }
  } else {
    fileName.value = '';
    file.value = null;
  }
}

function handleDragOver(ev: DragEvent) {
  ev.preventDefault();
  dragOver.value = true;
}

function handleDragLeave() {
  dragOver.value = false;
}
</script>

<template>
  <BoxModal
      :show="show"
      :loading="loading"
      :error-message="errorMessage"
      :onSubmit="onSubmit"
      :onCancel="onCancel"
  >
    <PartInput id="name" name="Nazwa" v-model="fileName" style="width: 100%;"/>
    <div class="file-input-container">
      <input type="file" id="file-input" name="Plik" @change="changeFile"/>
      <label for="file-input" class="file-label"
             :class="{'dragging': dragOver}"
             @dragover="handleDragOver"
             @dragenter="handleDragOver"
             @dragleave="handleDragLeave"
             @drop="handleDrop"
             tabindex="0">
        <Icon class="file-icon" name="material-symbols:upload-2-rounded"/>
        <span class="file-text">{{ file?.name || 'Wybierz plik' }}</span>
      </label>
    </div>
    <div v-if="loading" class="progress-bar">
      <PartProgress :value="progress" :max="100" :text="progress.toFixed(1)+'%'" bg-color="#aaa"
                    color="var(--accent-color)"/>
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
</template>

<style scoped>
.progress-bar {
  width: 100%;
  margin-top: 1rem;
  position: fixed;
  bottom: calc(100% + 1rem);
  background-color: var(--box-color);
  box-shadow: #000 .125rem .125rem 1rem .25rem;
  padding: 1rem;
  border-radius: 1rem;

  > * {
    border-radius: 1rem;
  }
}

.file-input-container {
  input {
    display: none;
  }

  .file-label {
    cursor: pointer;
    border-radius: 1rem;
    border: 2px solid var(--accent-color);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 12rem;
    width: 12rem;
    padding: 1rem;

    .file-icon {
      font-size: 5rem;
    }

    .file-text {
      max-width: 100%;
      white-space: normal;
      word-break: break-word;
      text-overflow: ellipsis;
      overflow: hidden;
    }

    &:hover {
      background-color: color-mix(in oklab, var(--accent-color), transparent 80%);
    }
  }
}

.dragging {
  background-color: color-mix(in oklab, var(--accent-color), transparent 50%);
  box-shadow: 0 4px 26px var(--accent-color);
  filter: brightness(1.12);
}
</style>