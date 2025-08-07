<script setup lang="ts">
import type {ApiResponse} from "~~/types/api";

const login = ref('')
const password = ref('')
const loading = ref(false)
const errorMessage = ref<ApiResponse | null >(null);
const auth = useAuthStore();

const handleLogin = async () => {
  try {
    loading.value = true;
    errorMessage.value = await auth.login({login: login.value, password: password.value});
    if (errorMessage.value.success) {
      await navigateTo('/')
    }
  } finally {
    loading.value = false
  }
}

useHead({
  title: "Assets - Logowanie"
})
</script>

<template>
  <main>
    <form @submit.prevent="handleLogin">
      <h1 v-if="loading">Logowanie...</h1>
      <h1 v-else>Zaloguj się</h1>
      <PartInput id="login" autocomplete="username" name="Login" v-model="login" :disabled="loading"/>
      <PartInput type="password" id="password" autocomplete="current-password" name="Hasło" v-model="password" :disabled="loading"/>
      <p v-if="errorMessage && !errorMessage.success" class="err-text">{{ errorMessage.detail }}</p>
      <PartButton type="submit" :disabled="loading">Zaloguj się</PartButton>
    </form>
  </main>
</template>

<style lang="scss" scoped>
.err-text {
  border: 2px solid white;
  background: #930000;
  padding: .5rem;
  border-radius: 9999rem;
  color: white;
}

main {
  padding-top: 10rem;

  form {
    background: var(--box-color);
    width: 30rem;
    display: flex;
    flex-direction: column;
    align-self: center;
    align-items: center;
    gap: 2rem;
    border-radius: 2rem;
    padding: 2rem;
  }
}
</style>