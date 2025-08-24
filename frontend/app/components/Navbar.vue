<script setup lang="ts">
import {useAuthStore} from "~/stores/auth.js";

const auth = useAuthStore()

const logout = async () => {
  profileMenu.value = false;
  await auth.logout();
  navigateTo("/login");
}

const profileMenu = ref(false);
const buttonOpenRef = ref(null);

function handleClickOutside(event: MouseEvent) {
  if (!buttonOpenRef.value.button.contains(event.target as Node)) {
    profileMenu.value = false;
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside);
});
onBeforeUnmount(() => {
  window.removeEventListener('mousedown', handleClickOutside);
});
</script>

<template>
  <div class="inner-box">
    <nav>
      <a href="/" class="title">
        <IconAssetBlue/>
        <p>AS</p></a>
      <div class="flex-fill"/>
      <div class="buttons">
        <p v-if="auth.loading">Ładowanie...</p>
        <template v-else>
          <template v-if="auth.isAuthenticated">
            <a href="/" tabindex="-1">
              <PartButton>Assety</PartButton>
            </a>
            <PartButton @click="profileMenu = !profileMenu" ref="buttonOpenRef">{{ auth.user.username }}</PartButton>
          </template>
          <template v-else>
            <nuxt-link href="/login">
              <PartButton>Zaloguj się</PartButton>
            </nuxt-link>
          </template>
        </template>
      </div>

      <transition name="profile-menu" mode="out-in">
        <div v-show="profileMenu" class="profile-menu">
          <RouterLink :to="{path: '/profile'}">Profil</RouterLink>
          <button @click="logout" style="background: var(--red-button-color)">Wyloguj się</button>
        </div>
      </transition>
    </nav>
  </div>
</template>

<style lang="scss" scoped>
.inner-box {
  padding: 1rem;
  width: 100%;
}

.profile-menu {
  position: absolute;
  top: calc(100% - 1rem);
  right: 0;
  display: flex;
  flex-direction: column;
  z-index: 11;
  border: 1px solid white;
  border-radius: 1rem;

  > * {
    background: var(--button-color);
    color: var(--text-color);
    font-size: 1rem;
    border: none;
    padding: 1rem;
    border-radius: 1rem;
    cursor: pointer;
    width: 14rem;
    text-align: left;

    &:first-child {
      border-bottom-right-radius: 0;
      border-bottom-left-radius: 0;
    }

    &:last-child {
      border-top-right-radius: 0;
      border-top-left-radius: 0;
    }

    &:hover {
      filter: brightness(75%);
    }
  }
}

.profile-menu-enter-active, .profile-menu-leave-active {
  transition: 300ms ease-out;
}

.profile-menu-enter-from, .profile-menu-leave-to {
  transform: translateY(-2rem);
  opacity: 0;
}

.profile-menu-enter-to, .profile-menu-leave-from {
  transform: translateY(0);
  opacity: 1;
}

nav {
  position: relative;
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

@media (max-width: 700px) {
  nav {
    border-radius: 0;
  }

  .inner-box {
    padding: 0;
  }
}

@media (max-width: 500px) {
  .title p {
    display: none;
  }
}
</style>