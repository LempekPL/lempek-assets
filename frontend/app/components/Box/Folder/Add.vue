<script setup lang="ts">
const props = defineProps<{
  show: boolean
  parentId?: string | null
}>()
const emit = defineEmits(['close','success'])

const loading = ref(false)
const errorMessage = ref('')
const folderName = ref('')

const config = useRuntimeConfig()

async function onSubmit() {
  loading.value = true
  errorMessage.value = ''
  try {
    await $fetch(config.public.apiBase + '/folder', {
      method: 'POST',
      credentials: 'include',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ name: folderName.value.trim(), parent: props.parentId || null })
    })
    folderName.value = ''
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
  folderName.value = ''
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
    <PartInput id="name" name="Nazwa" v-model="folderName" style="width: 100%;"/>
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--green-button-color)">Dodaj folder</PartButton>
    </template>
  </BoxModal>
</template>
