<script setup lang="ts">
const props = defineProps<{
  show: boolean
  type?: 'folder' | 'file'
  id: string
  name?: string
}>()
const emit = defineEmits(['close', 'success'])
const loading = ref(false)
const errorMessage = ref('')
const config = useRuntimeConfig()
const properType = computed(() => {
  if (props.type === 'file') {
    return 'plik';
  } else {
    return 'folder';
  }
})

async function onSubmit() {
  loading.value = true
  errorMessage.value = ''
  try {
    await $fetch(config.public.apiBase + '/' + props.type, {
      method: 'DELETE',
      credentials: 'include',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({id: props.id})
    })
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
    <p>Czy jesteś pewny że chcesz usunąć ten {{ properType }}?</p>
    <p class="item-name">{{ name }}</p>
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--red-button-color)">Usuń {{ properType }}
      </PartButton>
    </template>
  </BoxModal>
</template>

<style scoped>
.item-name {
  font-family: "JetBrains Mono", monospace, monospace;
  backdrop-filter: brightness(0.85);
  max-width: 100%;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  text-wrap: nowrap;
}
</style>