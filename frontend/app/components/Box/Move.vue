<script setup lang="ts">
import type {DraggedItem} from "~/pages/index.vue";

const props = defineProps<{
  show: boolean
  dragged?: DraggedItem | null
  newParent: string | null | undefined
}>()
const emit = defineEmits(['close', 'success'])

const loading = ref(false)
const errorMessage = ref('')

const config = useRuntimeConfig()

async function onSubmit() {
  if (props.newParent === undefined) return;
  loading.value = true
  errorMessage.value = ''
  try {
    await $fetch(`${config.public.apiBase}/${props.type}/move`, {
      method: 'PATCH',
      credentials: 'include',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({id: props.dragged?.item.id, new_parent: props.newParent})
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
    <p>Przenoszenie {{ dragged?.type === 'folder' ? 'folderu' : 'pliku' }} do {{ props.newParent }}</p>
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--blue-button-color)">Przenieś
        {{ dragged?.type === 'folder' ? 'folder' : 'plik' }}
      </PartButton>
    </template>
  </BoxModal>
</template>
