<script setup lang="ts">
import type {TypedItem} from "~~/types/api";

const props = defineProps<{
  show: boolean
  item: TypedItem | null
}>()
const emit = defineEmits(['close','success'])

const loading = ref(false)
const errorMessage = ref('')
const newItemName = ref<string | undefined>('')

watch(() => props.show, (show) => {
  if (show) newItemName.value = props.item?.item.name;
});

const config = useRuntimeConfig()

async function onSubmit() {
  if (!newItemName.value) return;
  loading.value = true
  errorMessage.value = ''
  try {
    await $fetch(`${config.public.apiBase}/${props.item?.type}/rename`, {
      method: 'PATCH',
      credentials: 'include',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ id: props.item?.item.id, name: newItemName.value.trim() })
    })
    newItemName.value = ''
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
  newItemName.value = ''
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
    <PartInput id="name" name="Nazwa" v-model="newItemName" style="width: 100%;"/>
    <template #action>
      <PartButton type="submit" :disabled="loading" style="background: var(--blue-button-color)">Edytuj nazwę</PartButton>
    </template>
  </BoxModal>
</template>
