<script setup lang="ts">
const props = defineProps<{
  show: boolean
  parentId?: string | null
}>()
const emit = defineEmits(['close', 'success'])

const loading = ref(false)
const errorMessage = ref('')
const file = ref<File | null>(null);
const fileName = ref('');

function changeFile(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    file.value = target.files.item(0);
    if (!fileName.value && file.value) {
      fileName.value = file.value.name;
    }
  }
}

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
    <input id="file" name="Plik" type="file" @change="changeFile" style="width: 100%;" />
    <div v-if="loading" class="progress-bar">
      <PartProgress :value="progress" :max="100" :text="progress.toFixed(1)+'%'" bg-color="#aaa" color="var(--accent-color)"/>
    </div>
    <template #cancel>
      <PartButton type="button" @click="onCancel" :style="{ backgroundColor: loading ? 'var(--red-button-color)' : '' }">Anuluj</PartButton>
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
</style>