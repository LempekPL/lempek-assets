<script setup lang="ts">
import type {ApiResponse, RefreshToken, UserAll} from "~~/types/api";

const config = useRuntimeConfig();

const userpass = reactive({
  login: '',
  password: '',
  new_password: '',
  message: null as ApiResponse | null,
  loading: false
})
const nameChange = reactive({
  login: '',
  password: '',
  new_username: '',
  message: null as ApiResponse | null,
  loading: false
})
const createNewUser = reactive({login: '', password: '', message: null as ApiResponse | null, loading: false})

const auth = useAuthStore();
const devicesClosed = ref(true)

const handleUpdatePassword = async () => {
  try {
    userpass.loading = true;
    userpass.message = await auth.changePassword({
      current_password: userpass.password,
      new_password: userpass.new_password
    });
    if (userpass.message.success) {
      userpass.password = '';
      userpass.new_password = '';
      await navigateTo('/profile');
    }
  } finally {
    userpass.loading = false
  }
}

const handleUpdateName = async () => {
  try {
    nameChange.loading = true;
    nameChange.message = await auth.changeUsername({
      password: nameChange.password,
      new_username: nameChange.new_username
    });
    if (nameChange.message.success) {
      nameChange.password = '';
      nameChange.new_username = '';
      await navigateTo('/profile');
    }
  } finally {
    nameChange.loading = false
  }
}

const handleCreateUser = async () => {
  try {
    createNewUser.loading = true;
    createNewUser.message = await $fetch<ApiResponse>(config.public.apiBase + '/user/create', {
      method: 'POST',
      credentials: 'include',
      body: {login: createNewUser.login, password: createNewUser.password}
    });
    if (createNewUser.message.success) {
      createNewUser.login = '';
      createNewUser.password = '';
      await navigateTo('/profile');
    }
  } catch (error: any) {
    if (error?.data) {
      createNewUser.message = error.data;
    } else {
      createNewUser.message = {
        success: false,
        detail: 'Nie udało się stworzyć użytkownika (błąd sieci).',
        err_id: null
      }
    }
  } finally {
    createNewUser.loading = false
  }
}

async function removeToken(id: string) {
  try {
    await $fetch<ApiResponse>(config.public.apiBase + '/user/token', {
      method: 'DELETE',
      credentials: 'include',
      body: {id}
    });
  } catch (error: any) {
  //   if (error?.data) {
  //     createNewUser.message = error.data;
  //   } else {
  //     createNewUser.message = {
  //       success: false,
  //       detail: 'Nie udało się stworzyć użytkownika (błąd sieci).',
  //       err_id: null
  //     }
  //   }
  }
  refreshDevices();
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
  title: "AS Profil"
})
</script>

<template>
  <HeaderBox width="min(100%, 60vh)">
    <template #header>Profil</template>
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
      <p>{{ new Date(user.created_at.substring(0, 23) + "Z").toLocaleString(undefined, {day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit"}) }}</p>
      <p>Zaktualizowany:</p>
      <p>{{ new Date(user.created_at.substring(0, 23) + "Z").toLocaleString(undefined, {day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit"}) }}</p>
    </div>
  </HeaderBox>
  <HeaderBox width="min(100%, 60vh)">
    <template #header>
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
      <form @submit.prevent="() => removeToken(token.id)" v-for="token in deviceTokens" :key="token.id"
            class="device-token">
        <div>
          <p>{{ new Date(token.expires_at.substring(0, 23) + "Z").toLocaleString(undefined, {day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit"}) }}</p>
          <p class="device-place" v-if="token.country || token.region || token.city">{{ token.country }}, {{ token.region }}, {{ token.city }}</p>
          <p class="device-place" v-else>nie wiadoma lokacja</p>
        </div>
        <PartButton type="submit" class="device-button">Usuń token</PartButton>
      </form>
    </div>
  </HeaderBox>
  <HeaderBox v-if="auth.user?.admin" width="min(100%, 60vh)">
    <template #header>Stwórz użytkownika</template>
    <form @submit.prevent="handleCreateUser">
      <PartInput id="nu_name" name="Login użytkownika" v-model="createNewUser.login" required="required"
                 :disabled="createNewUser.loading"/>
      <PartInput type="password" id="nu_pass" name="Hasło użytkownika" v-model="createNewUser.password"
                 required="required" :disabled="createNewUser.loading"/>
      <BoxError v-if="createNewUser.message && !createNewUser.message.success" :message="createNewUser.message.detail"/>
      <BoxOk v-if="createNewUser.message && createNewUser.message.success" message="Stworzono użytkownika"/>
      <PartButton type="submit" :disabled="createNewUser.loading">Stwórz użytkownika</PartButton>
    </form>
  </HeaderBox>
  <HeaderBox width="min(100%, 60vh)">
    <template #header>Zmień nazwę</template>
    <form @submit.prevent="handleUpdateName">
      <PartInput id="nn_login" name="Login" v-model="nameChange.login" hidden="hidden" required="not"/>
      <PartInput id="nn_name" name="Nowa Nazwa" v-model="nameChange.new_username" required="required"
                 :disabled="nameChange.loading"/>
      <PartInput type="password" id="nn_pass" name="Hasło" v-model="nameChange.password" required="required"
                 :disabled="nameChange.loading"/>
      <BoxError v-if="nameChange.message && !nameChange.message.success" :message="nameChange.message.detail"/>
      <BoxOk v-if="nameChange.message && nameChange.message.success" message="Zmieniono nazwę"/>
      <PartButton type="submit" :disabled="nameChange.loading">Zmień nazwę</PartButton>
    </form>
  </HeaderBox>
  <HeaderBox width="min(100%, 60vh)">
    <template #header>Zmień Hasło</template>
    <form @submit.prevent="handleUpdatePassword">
      <PartInput id="login" autocomplete="username" name="Login" v-model="userpass.login" hidden="hidden"
                 required="not"/>
      <PartInput type="password" id="password" autocomplete="current-password" name="Stare Hasło"
                 v-model="userpass.password"
                 :disabled="userpass.loading"/>
      <PartInput type="password" id="new-password" autocomplete="new-password" name="Nowe Hasło"
                 v-model="userpass.new_password"
                 :disabled="userpass.loading"/>
      <BoxError v-if="userpass.message && !userpass.message.success" :message="userpass.message.detail"/>
      <BoxOk v-if="userpass.message && userpass.message.success" message="Ustawiono nowe hasło"/>
      <PartButton type="submit" :disabled="userpass.loading">Zaktualizuj hasło</PartButton>
    </form>
  </HeaderBox>
</template>

<style scoped>
.device-tokens {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 100%;

  .device-token {
    display: flex;
    flex-direction: row;
    background: var(--background-color);
    border-radius: 1rem;
    padding: 1rem;
    gap: 1rem;

    > div {
      display: flex;
      flex-direction: column;
      gap: .25rem;
    }

    .device-place {
      font-size: 0.9rem;
      filter: opacity(.75);
    }

    .device-button {
      background: var(--red-button-color);
    }
  }
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

form {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1rem;

  > * {
    width: min(100%, 70vh);

    &:last-child {
      align-self: flex-end;
      width: 12rem;
    }
  }
}

.profile-info {
  display: grid;
  grid-template-columns: 8rem 1fr;
  gap: 0.5rem 1rem;
}
</style>
