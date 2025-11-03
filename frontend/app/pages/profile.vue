<script setup lang="ts">
import type {ApiResponse, RefreshToken, UserAll} from "~~/types/api";

const config = useRuntimeConfig();

const login = ref('')
const password = ref('')
const new_password = ref('')
const loading = ref(false)
const message = ref<ApiResponse | null>(null);
const auth = useAuthStore();
const devicesClosed = ref(true)

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

const {
  data: deviceTokens,
  refresh: refreshDevices
} = await useFetch<RefreshToken[]>(() => config.public.apiBase + `/user/tokens`, {
  method: 'GET',
  credentials: 'include',
  headers: {'Content-Type': 'application/json'},
});

const user = await $fetch<UserAll>(config.public.apiBase + `/user/all`, {
  method: 'GET',
  credentials: 'include',
  headers: {'Content-Type': 'application/json'},
})


useHead({
  title: "AS - Profil"
})
</script>

<template>
  <ProfileBox width="min(100%, 60vh)">
    <template #name>Profil</template>
    <div class="profile-info">
      <p>UUID:</p>
      <p>{{ user.id }}</p>
      <p>Nazwa:</p>
      <p>{{ user.username }}</p>
      <p>Login:</p>
      <p>{{ user.login }}</p>
      <p>Admin:</p>
      <p>{{ user.admin ? 'tak' : 'nie' }}</p>
      <p>Stworzony:</p>
      <p>{{ new Date(user.created_at.substring(0, 23) + "Z").toLocaleString() }}</p>
      <p>Zaktualizowany:</p>
      <p>{{ new Date(user.created_at.substring(0, 23) + "Z").toLocaleString() }}</p>
    </div>
  </ProfileBox>
  <ProfileBox width="min(100%, 60vh)">
    <template #name>
      <span style="cursor: pointer"
            @click="devicesClosed = !devicesClosed">Urządzenia [{{ deviceTokens?.length }}] <Icon
          class="icon"
          :name="devicesClosed ? 'material-symbols:arrow-downward-rounded' : 'material-symbols:arrow-upward-rounded'"/>
      </span>
    </template>
    <div class="device-closed" v-if="devicesClosed" @click="devicesClosed = false">
      <Icon class="icon" name="material-symbols:circle"/>
      <Icon class="icon" name="material-symbols:circle"/>
      <Icon class="icon" name="material-symbols:circle"/>
    </div>
    <div v-else class="device-tokens">
      <div v-for="token in deviceTokens" :key="token.id" class="device-token">
        <p>{{ token.id }}</p>
        <p>{{ token.user_agent }}</p>
        <p>{{ token.expires_at }}</p>
      </div>
    </div>
  </ProfileBox>
  <ProfileBox width="min(100%, 60vh)">
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
  grid-template-columns: 8rem 1fr;
  gap: 0.5rem 1rem;
}

.device-closed {
  width: 100%;
  padding: .5rem;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

.device-tokens {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
