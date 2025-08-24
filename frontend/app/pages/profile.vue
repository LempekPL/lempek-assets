<script setup lang="ts">
import type {ApiResponse} from "~~/types/api";

const login = ref('')
const password = ref('')
const new_password = ref('')
const loading = ref(false)
const message = ref<ApiResponse | null>(null);
const auth = useAuthStore();

const handleUpdatePassword = async () => {
  try {
    loading.value = true;
    message.value = await auth.changePassword({current_password: password.value, new_password: new_password.value});
    if (message.value.success) {
      password.value = '';
      new_password.value = '';
      await navigateTo('/profile');
    }
  } finally {
    loading.value = false
  }
}

useHead({
  title: "AS - Profil"
})
</script>

<template>
  <ProfileBox width="min(100%, 50vh)">
    <template #name>Profil</template>
    <div class="profile-info">
      <p>UUID:</p>
      <p>{{ auth.user.user_id }}</p>
      <p>Nazwa:</p>
      <p>{{ auth.user.username }}</p>
      <p>Login:</p>
      <p>{{ auth.user.login }}</p>
      <p>Admin:</p>
      <p>{{ auth.user.admin ? 'tak' : 'nie' }}</p>
    </div>
  </ProfileBox>
  <ProfileBox width="min(100%, 50vh)">
    <template #name>Zmień Hasło</template>
    <form @submit.prevent="handleUpdatePassword">
      <PartInput id="login" autocomplete="username" name="Login" v-model="login" hidden="hidden" required="not"/>
      <PartInput type="password" id="password" autocomplete="current-password" name="Stare Hasło" v-model="password"
                 :disabled="loading"/>
      <PartInput type="password" id="new-password" autocomplete="new-password" name="Nowe Hasło" v-model="new_password"
                 :disabled="loading"/>
      <BoxError v-if="message && !message.success" :message="message.detail"/>
      <BoxOk v-if="message && message.success" message="Ustawiono nowe hasło"/>
      <PartButton type="submit" :disabled="loading">Zaktualizuj hasło</PartButton>
    </form>
  </ProfileBox>
</template>

<style scoped>
form {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1rem;

  > * {
    width: min(100%, 70vh);

    &:last-child {
      align-self: flex-end;
      width: 10rem;
    }
  }
}

.profile-info {
  display: grid;
  grid-template-columns: 4rem 1fr;
  gap: 0.5rem 1rem;
}
</style>
