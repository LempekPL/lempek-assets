<script setup lang="ts">
import type {NuxtError} from '#app'

const props = defineProps({
  error: Object as () => NuxtError,
})
</script>

<template>
  <Navbar/>
  <main>
    <div>
      <p class="error-code">{{ error.statusCode }}</p>
      <p>{{ error?.message }}</p>
      <NuxtLink to="/">
        <PartButton>
          Go back home
        </PartButton>
      </NuxtLink>
    </div>
  </main>
</template>

<style scoped>
main {
  padding-top: 8rem;

  .error-code {
    font-size: 4rem;
  }

  div {
    background: var(--box-color);
    width: 20rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    border-radius: 2rem;
    padding: 2rem;
  }
}
</style>