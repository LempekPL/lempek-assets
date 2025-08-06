<script setup lang="ts">
import {useAuthStore} from "~/stores/auth.js";
import AddFolderBox from "~/components/AddFolderBox.vue";
import {type Folder} from "~~/types/api";
import {useRouter} from 'vue-router';

const router = useRouter();
const route = useRoute();
const auth = useAuthStore();
const parentId = computed(() => route.query.parent);
const config = useRuntimeConfig();
const {
  data: folders,
  pending,
  error
} = await useFetch<Folder[]>(() => config.public.apiBase + "/folders?parent=" + (parentId.value ?? ''), {
  method: 'GET',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
  watch: [parentId]
});

const menuBoxRef = ref<HTMLElement | null>(null);
const menuBoxVisible = ref(false);
const menuBoxX = ref(0);
const menuBoxY = ref(0);
const selectedFolder = ref<Folder | null>(null);

function openMenuBox(event: MouseEvent, folder: Folder | null) {
  event.preventDefault();
  menuBoxVisible.value = true;
  menuBoxX.value = event.clientX;
  menuBoxY.value = event.clientY;
  selectedFolder.value = folder;
}

function handleClickOutside(event: MouseEvent) {
  if (!menuBoxVisible.value) return;
  if (menuBoxRef.value && !menuBoxRef.value.contains(event.target as Node)) {
    menuBoxVisible.value = false;
    selectedFolder.value = null;
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside);
});
onBeforeUnmount(() => {
  window.removeEventListener('mousedown', handleClickOutside);
});

const displayBigBox = ref(false);

function openAddFolderBox() {
  menuBoxVisible.value = false;
  displayBigBox.value = true;
}

function enterFolder(folder: Folder) {
  router.push({path: '/', query: {parent: folder.id}});
}
</script>

<template>
  <div v-if="pending">Loading folders...</div>

  <div v-if="error" class="error">
    Error loading folders: {{ error.message }}
  </div>

  <div v-else @contextmenu.prevent="openMenuBox($event, null)" class="main-box">
    <div class="items-grid">
      <div v-for="folder in folders" :key="folder.id" class="item"
           @contextmenu.prevent.stop="openMenuBox($event, folder)" @dblclick="enterFolder(folder)">
        <Icon name="fa6-solid:folder"/>
        <p>{{ folder.name }}</p>
      </div>
    </div>

    <div v-if="menuBoxVisible" class="menu-box" ref="menuBoxRef">
      <button v-if="selectedFolder">
        <Icon name="fa6-solid:folder-plus"/>
        <span>Edytuj nazwę</span>
      </button>
      <button v-if="selectedFolder">
        <Icon name="fa6-solid:folder-plus"/>
        <span>Usuń folder</span>
      </button>
      <div v-if="selectedFolder"/>
      <button @click="openAddFolderBox">
        <Icon name="fa6-solid:folder-plus"/>
        <span>Nowy folder</span>
      </button>
    </div>
  </div>

  <AddFolderBox :show="displayBigBox" @close="displayBigBox = false"/>
</template>

<style scoped lang="scss">
.items-grid {
  padding-top: var(--body-padding);
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
  gap: 1rem;


  .item {
    user-select: none;
    padding: 2rem;
    height: 13rem;
    border-radius: 1rem;
    background-color: var(--box-color);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    gap: 2rem;
    text-align: center;
    cursor: pointer;

    &:active {
      transform: scale(0.95);
    }

    span {
      padding: 3rem;
    }

    p {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 100%;
      line-height: 1;
      padding-bottom: 4px;
    }
  }
}

.menu-box {
  position: absolute;
  top: calc(v-bind(menuBoxY) * 1px - 134px);
  left: calc(v-bind(menuBoxX) * 1px - 1rem);
  display: flex;
  flex-direction: column;

  div {
    border-bottom: var(--background-color) solid 2px;
  }

  button {
    --button-height: 3rem;
    font-size: 1rem;
    cursor: pointer;
    border: none;
    background: var(--button-color);
    padding: 0;
    height: var(--button-height);
    width: 14rem;
    text-align: left;
    display: flex;
    justify-content: left;
    align-items: center;

    span:first-child {
      padding: .75rem;
      width: var(--button-height);
    }

    &:first-child {
      border-radius: 1rem 1rem 0 0;
    }

    &:last-child {
      border-radius: 0 0 1rem 1rem;
    }

    &:only-child {
      border-radius: 1rem;
    }

    &:hover {
      filter: brightness(80%);
    }
  }
}

.main-box {
  position: relative;
  width: 100%;
  flex: 1;
}

.error {
  color: #dc3545;
}
</style>