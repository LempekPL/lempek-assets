<script setup>

const props = defineProps({
  id: {
    type: String,
    required: true
  },
  name: {
    type: String,
    required: true
  },
  disabled: {
    type: Boolean,
    required: false
  },
  modelValue: {
    type: [String, Number],
    default: ''
  },
  type: {
    type: String,
    default: 'text'
  },
  autocomplete: {
    type: String,
    required: false
  }
});

const emit = defineEmits(['update:modelValue']);
</script>

<template>
  <div class="input-text">
    <input
        :type="type"
        :id="props.id"
        :name="props.id"
        :placeholder="props.id"
        :disabled="disabled"
        :autocomplete="props.autocomplete"
        @input="$emit('update:modelValue', $event.target.value)"
        required
    />
    <label :for="props.id">{{ name }}</label>
  </div>
</template>

<style scoped>
div {
  position: relative;
  z-index: 1;
  width: 80%;
  border: 2px solid var(--accent-color);
  border-radius: 99999rem;

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

  &:focus-within > label, > input:not(:placeholder-shown) + label {
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

    &::placeholder {
      color: var(--box-color);
      user-select: none;
    }

    &:is(:-webkit-autofill, :autofill) {
      border-radius: 99999rem;
    }
  }
}
</style>