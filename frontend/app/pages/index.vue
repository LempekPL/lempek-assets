<!--suppress HtmlUnknownTarget -->
<script setup lang="ts">
import PartMiniMenu from "~/components/part/MiniMenu.vue";
import type {Folder, UuidName} from "~~/types/api";
import {useRouter} from 'vue-router';

const router = useRouter();
const route = useRoute();
const parentId = computed(() => route.query.parent as string | null);
const config = useRuntimeConfig();

const {
  data: folders,
  pending,
  error,
  refresh: refreshFolders
} = await useFetch<Folder[]>(() => config.public.apiBase + "/folders?parent=" + (parentId.value ?? ''), {
  method: 'GET',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
  watch: [parentId]
});

const {
  data: folderPath,
} = await useFetch<UuidName[]>(() => config.public.apiBase + "/folder/path?id=" + (parentId.value ?? ''), {
  method: 'GET',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
  watch: [parentId]
});

const menuRef = ref<InstanceType<typeof PartMiniMenu> | null>(null);
const selectedFolder = ref<Folder | null>(null);

function openMenuBox(event: MouseEvent, folder: Folder | null) {
  event.preventDefault();
  menuRef.value?.open(event.clientX, event.clientY);
  selectedFolder.value = folder;
}

function handleClickOutside(event: MouseEvent) {
  if (!menuRef.value?.isOpen()) return;
  if (!menuRef.value.contains(event.target as Node)) {
    menuRef.value.close();
    selectedFolder.value = null;
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside);
});
onBeforeUnmount(() => {
  window.removeEventListener('mousedown', handleClickOutside);
});

const addFolderBox = ref(false);
const deleteFolderBox = ref(false);

function enterFolder(id: string | null) {
  router.push({path: '/', query: {parent: id}});
}

function handleSuccess() {
  addFolderBox.value = false;
  deleteFolderBox.value = false;
  refreshFolders();
}

useHead({
  title: "Assets",
})
</script>

<template>
  <div class="option-box">
    <div>
      <div class="path-text">
        <RouterLink :to="{ path: '/' }">/</RouterLink>
        <template v-for="pathItem in folderPath" :key="pathItem.id">
          <RouterLink :to="{ path: '/', query: { parent: pathItem.id } }">{{ pathItem.name }}</RouterLink>
          <RouterLink :to="{ path: '/', query: { parent: pathItem.id } }">/</RouterLink>
        </template>
      </div>
    </div>
  </div>


  <transition name="fade" mode="out-in">
    <div v-if="error" class="error">
      Error: {{ error.message }}
    </div>

    <div v-else-if="pending" class="loading-folders">
      <div>
        <p>Ładowanie folderów...</p>
      </div>
    </div>

    <div v-else @contextmenu.prevent="openMenuBox($event, null)" class="main-box">
      <div class="items-grid">
        <div v-for="folder in folders" :key="folder.id" class="item"
             @contextmenu.prevent.stop="openMenuBox($event, folder)" @dblclick="enterFolder(folder.id)">
          <Icon name="fa6-solid:folder"/>
          <p>{{ folder.name }}</p>
        </div>
      </div>
    </div>
  </transition>

  <PartMiniMenu ref="menuRef" class="menu-part">
    <button v-if="selectedFolder">
      <Icon name="fa6-solid:folder-plus"/>
      <span>Edytuj nazwę</span>
    </button>
    <button v-if="selectedFolder" @click="() => {menuRef?.close(); deleteFolderBox = true}">
      <Icon name="fa6-solid:folder-plus"/>
      <span>Usuń folder</span>
    </button>
    <div v-if="selectedFolder"/>
    <button @click="() => {menuRef?.close(); addFolderBox = true}">
      <Icon name="fa6-solid:folder-plus"/>
      <span>Nowy folder</span>
    </button>
  </PartMiniMenu>

  <FolderBoxAdd
      :show="addFolderBox"
      @close="addFolderBox = false"
      @success="handleSuccess"
      :parent-id="parentId || undefined"/>

  <FolderBoxDelete
      :show="deleteFolderBox"
      @close="deleteFolderBox = false"
      @success="handleSuccess"
      :folder-id="selectedFolder?.id ?? ''"
      :folder-name="selectedFolder?.name"/>
</template>

<style scoped lang="scss">
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.25s;
  display: inline;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
  display: inline;
}

.fade-enter-to, .fade-leave-from {
  opacity: 1;
  display: block;
}


.loading-folders {
  padding-top: var(--body-padding);
  justify-items: center;

  div {
    background: var(--box-color);
    padding: 1rem;
    border-radius: 1rem;
  }
}

.option-box {
  padding-top: var(--body-padding);
  width: 100%;
  justify-items: center;

  div {
    background: var(--box-color);
    padding: .2rem .5rem;
    border-radius: .5rem;
    display: flex;
    flex-direction: row;
    width: 90%;

    .path-text > * {
      padding: .5rem;
      cursor: pointer;
      color: var(--text-color);
      background: none;
      border: none;
      font-size: 1rem;

      &:hover {
        text-decoration: underline;
        transform: scale(1.05);
      }

      &:active {
        text-decoration: underline;
        transform: scale(0.95);
      }
    }
  }
}

.menu-part {
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

.main-box {
  position: relative;
  width: 100%;
  flex: 1;
}

.error {
  color: #dc3545;
}
</style>