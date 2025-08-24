<script setup lang="ts">
import type {ApiResponse} from "~~/types/api";

const login = ref('')
const password = ref('')
const loading = ref(false)
const message = ref<ApiResponse | null>(null);
const auth = useAuthStore();

const handleLogin = async () => {
  try {
    loading.value = true;
    message.value = await auth.login({login: login.value, password: password.value});
    if (message.value.success) {
      await navigateTo('/')
    }
  } finally {
    loading.value = false
  }
}

useHead({
  title: "AS - Logowanie"
})
</script>

<template>
  <main>
    <form @submit.prevent="handleLogin">
      <h1 v-if="loading">Logowanie...</h1>
      <h1 v-else>Zaloguj się</h1>
      <PartInput id="login" autocomplete="username" name="Login" v-model="login" :disabled="loading"/>
      <PartInput type="password" id="password" autocomplete="current-password" name="Hasło" v-model="password"
                 :disabled="loading"/>
      <BoxError v-if="message && !message.success" :message="message.detail"/>
      <PartButton type="submit" :disabled="loading">Zaloguj się</PartButton>
    </form>
  </main>
</template>

<style lang="scss" scoped>
main {
  padding-top: 8rem;

  form {
    background: var(--box-color);
    width: 30rem;
    display: flex;
    flex-direction: column;
    align-self: center;
    align-items: center;
    gap: 1.5rem;
    border-radius: 2rem;
    padding: 2rem;
  }
}
</style>