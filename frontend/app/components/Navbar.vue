<script setup>
import {useAuthStore} from "~/stores/auth.js";

const auth = useAuthStore()

const logout = async () => {
  await auth.logout();
  navigateTo("/login");
}
</script>

<template>
  <nav>
    <a href="/" class="title"><IconAssetBlue /><p>Assety</p></a>
    <div class="flex-fill"/>
    <div class="buttons">
      <p v-if="auth.loading">Ładowanie...</p>
      <template v-else>
        <template v-if="auth.isAuthenticated">
          <a href="/">
            <PartButton>Assety</PartButton>
          </a>
          <PartButton @click="logout">Wyloguj się</PartButton>
<!--          <p class="logged-name">Zalogowano jako {{ auth.user.login }}</p>-->
        </template>
        <template v-else>
          <nuxt-link href="/login">
            <PartButton>Zaloguj się</PartButton>
          </nuxt-link>
          <nuxt-link href="/register">
            <PartButton>Zarejestruj się</PartButton>
          </nuxt-link>
        </template>
      </template>
    </div>
  </nav>
</template>

<style lang="scss" scoped>

nav {
  width: 100%;
  color: var(--text-color);
  background: var(--box-color);
  padding: 2rem;
  border-radius: 2rem;
  display: flex;
  flex-direction: row;
  align-items: center;
  //position: sticky;
  //top: 1rem;
  //z-index: 999;

  .title {
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;

    & p {
      font-weight: bold;
      font-size: 2rem;
      padding-left: 5rem;
    }

    & svg {
      position: absolute;
      left: -1rem;
      height: 5rem;
    }
  }

  .buttons {
    display: flex;
    position: relative;
    gap: 1rem;
    flex-direction: row;
  }
}
</style>