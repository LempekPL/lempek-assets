<script setup lang="ts">
const login = ref('')
const password = ref('')
const loading = ref(false)
const errorMessage = ref<{success: boolean, message: string}>(null)
const auth = useAuthStore()

const handleLogin = async () => {
  try {
    loading.value = true
    errorMessage.value = ''

    errorMessage.value = await auth.register({login: login.value, password: password.value});
    if (errorMessage.value.success) {
      await navigateTo('/')
    }
  } catch (error) {
    console.log(error);
    errorMessage.value.message = 'Nie udało się zarejestrować! :('
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <main>
    <form @submit.prevent="handleLogin">
      <h1 v-if="loading">Rejestrowanie...</h1>
      <h1 v-else>Zarejestruj się</h1>
      <PartInput id="login" name="Login" v-model="login" :disabled="loading"/>
      <PartInput type="password" id="password" name="Hasło" v-model="password" :disabled="loading"/>
      <p v-if="errorMessage" class="err-text">{{ errorMessage.message }}</p>
      <PartButton type="submit" :disabled="loading">Zarejestruj się</PartButton>
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