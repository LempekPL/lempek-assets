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
    file.value = target.files[0];
    if (!fileName.value) {
      fileName.value = file.value.name;
    }
  } else {
    fileName.value = null;
    file.value = null;
  }
}

const config = useRuntimeConfig()
async function onSubmit() {
  if (!file.value) {
    errorMessage.value = 'Plik nie został wybrany.'
    return
  }
  loading.value = true
  errorMessage.value = ''
  try {
    const formData = new FormData();
    formData.append('file', file.value);
    formData.append('name', fileName.value);
    if (props.parentId) formData.append('folder', props.parentId);
    // if (props.parentId) formData.append('folder', props.parentId);
    await $fetch(config.public.apiBase + '/upload', {
      method: 'POST',
      credentials: 'include',
      body: formData
    })
    fileName.value = '';
    file.value = null;
    emit('success')
  } catch (err: any) {
    errorMessage.value = err?.data?.detail || 'Błąd'
  } finally {
    loading.value = false
  }
}

function onCancel() {
  if (loading.value) return
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
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--green-button-color)">Wyślij plik
      </PartButton>
    </template>
  </BoxModal>
</template>
