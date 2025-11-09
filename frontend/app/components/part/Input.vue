<script setup lang="ts">
defineProps<{
  id: string
  name: string
  disabled?: boolean
  modelValue: any
  type?: string
  autocomplete?: string
  required?: 'required' | 'not'
  placeholder?: string
}>();

const emit = defineEmits(['update:modelValue']);
</script>

<template>
  <div class="input-text">
    <input
        :type="type ?? 'text'"
        :id="id"
        :name="name"
        :placeholder="placeholder"
        :disabled="disabled"
        :autocomplete="autocomplete"
        :model-value="modelValue"
        :value="modelValue"
        @input="$emit('update:modelValue', $event.target.value)"
        :required="required === 'required'"/>
    <label :for="id">{{ name }}</label>
  </div>
</template>

<style scoped>
div {
  position: relative;
  z-index: 1;
  width: 80%;
  outline: 1px solid var(--button-color);
  border-radius: 99999rem;
  transition: 100ms;
  transition-timing-function: cubic-bezier(.4, 0, .2, 1);

  > label {
    align-content: center;
    position: absolute;
    top: .1rem;
    transform: translateY(50%);
    left: 1rem;
    color: var(--text-color);
    transition: 150ms;
    transition-timing-function: cubic-bezier(.4, 0, .2, 1);
  }

  &:focus-within > label {
    color: var(--accent-color);
  }

  &:focus-within {
    outline-color: var(--accent-color);
    outline-width: 2px;
  }

  &:focus-within > label, > input:is(:not([value=""]), :placeholder-shown) + label {
    position: absolute;
    background: var(--box-color);
    top: 0;
    left: 1.1rem;
    transform: translateY(-50%);
    font-size: .75rem;
    padding: 0 .25rem;
  }

  > input {
    display: inline-block;
    width: 100%;
    background: transparent;
    outline: none;
    font-size: 1rem;
    z-index: 1;
    color: var(--text-color);
    padding: .75rem;
    border: none;

    &:disabled {
      cursor: not-allowed;
    }

    &:is(:-webkit-autofill, :autofill) {
      border-radius: 99999rem;
    }
  }
}
</style>