<script setup lang="ts">
const props = defineProps<{
  show: boolean
  folderId: string
  folderName: string
}>()
const emit = defineEmits(['close','success'])

const loading = ref(false)
const errorMessage = ref('')
const newFolderName = ref('')

watch(() => props.show, (show) => {
  if (show) newFolderName.value = props.folderName;
});

const config = useRuntimeConfig()

async function onSubmit() {
  loading.value = true
  errorMessage.value = ''
  try {
    await $fetch(config.public.apiBase + '/folder', {
      method: 'PATCH',
      credentials: 'include',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ id: props.folderId, name: newFolderName.value.trim() })
    })
    newFolderName.value = ''
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
  newFolderName.value = ''
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
    <PartInput id="name" name="Nazwa" v-model="newFolderName" style="width: 100%;"/>
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--blue-button-color)">Edytuj nazwę</PartButton>
    </template>
  </BoxModal>
</template>
