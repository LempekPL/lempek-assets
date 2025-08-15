<!--suppress HtmlUnknownTarget -->
<script setup lang="ts">
import PartMiniMenu from "~/components/part/MiniMenu.vue";
import type {Folder, File, UuidName} from "~~/types/api";
import {useRouter} from 'vue-router';

const router = useRouter();
const route = useRoute();
const parentId = computed(() => route.query.parent as string | null);
const orderInfo = computed(() => route.query.order as string | null);
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
  data: files,
  refresh: refreshFiles
} = await useFetch<File[]>(() => config.public.apiBase + "/files?parent=" + (parentId.value ?? ''), {
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
const selectedItem = ref<Folder | File | null>(null);
const selectedType = ref<'folder' | 'file' | null>(null);

function openMenuBox(event: MouseEvent, item: Folder | File | null, type: 'file' | 'folder' | null) {
  event.preventDefault();
  menuRef.value?.open(event.clientX, event.clientY);
  selectedItem.value = item;
  selectedType.value = type;
}

function handleClickOutside(event: MouseEvent) {
  if (!menuRef.value?.isOpen()) return;
  if (!menuRef.value.contains(event.target as Node)) {
    menuRef.value.close();
    selectedItem.value = null;
    selectedType.value = null;
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
const editFolderBox = ref(false);
const addFileBox = ref(false);

function enterFolder(id: string | null) {
  router.push({path: '/', query: {parent: id}});
}

function handleSuccess() {
  addFolderBox.value = false;
  deleteFolderBox.value = false;
  editFolderBox.value = false;
  addFileBox.value = false;
  refreshFolders();
  refreshFiles();
}

useHead({
  title: "Assety",
})

const folderPathSpliced = computed(() => {
  if (folderPath.value && folderPath.value.length > 7) {
    return folderPath.value.slice(folderPath.value.length - 7);
  } else {
    return folderPath.value || [];
  }
})
</script>

<template>
  <div class="option-box">
    <div>
      <div class="path-text">
        <RouterLink :to="{ path: '/' }">/</RouterLink>
        <template v-for="(pathItem, idx) in folderPathSpliced" :key="pathItem.id">
          <RouterLink :to="{ path: '/', query: { parent: pathItem.id } }">
            <template v-if="(folderPath?.length ?? 0) > 6 && idx === 0">
              ...
            </template>
            <template v-else-if="(folderPath?.length ?? 0) > 6 && idx > 0 || (folderPath?.length ?? 0) <= 6">
              {{ pathItem.name }}
            </template>
          </RouterLink>
          <RouterLink :to="{ path: '/', query: { parent: pathItem.id } }">/</RouterLink>
        </template>
      </div>
    </div>
  </div>


  <transition name="fade" mode="out-in">
    <div v-if="error" class="error">
      Error: {{ error.message }}
    </div>

    <div v-else-if="pending" class="default-box">
      <div>
        <p>Ładowanie folderów...</p>
      </div>
    </div>

    <div v-else @contextmenu.prevent="openMenuBox($event, null)" class="main-box">
      <div class="items-grid">
        <div v-for="folder in folders" :key="folder.id" class="item"
             @contextmenu.prevent.stop="openMenuBox($event, folder)" @dblclick="enterFolder(folder.id)">
          <Icon name="material-symbols:folder-rounded"/>
          <p>{{ folder.name }}</p>
        </div>
        <div v-for="file in files" :key="file.id" class="item"
             @contextmenu.prevent.stop="openMenuBox($event, file)" @dblclick="enterFolder(file.id)">
          <Icon name="material-symbols:unknown-document-rounded"/>
          <p>{{ file.name }}</p>
        </div>
      </div>

      <div v-show="folders?.length === 0 && files?.length === 0" class="default-box">
        <div>
          <p>Brak przedmiotów w tym folderze</p>
        </div>
      </div>
    </div>
  </transition>

  <PartMiniMenu ref="menuRef" class="menu-part">
    <button v-if="selectedType == 'folder'" @click="() => {menuRef?.close(); editFolderBox = true}">
      <Icon name="material-symbols:folder-managed"/>
      <span>Edytuj nazwę</span>
    </button>
    <button v-if="selectedType == 'folder'" @click="() => {menuRef?.close(); deleteFolderBox = true}">
      <Icon name="material-symbols:folder-delete-rounded"/>
      <span>Usuń folder</span>
    </button>
    <div v-if="selectedType == 'folder'"/>

    <button v-if="selectedType == 'file'" @click="() => {menuRef?.close(); editFolderBox = true}">
      <Icon name="material-symbols:edit-square-rounded"/>
      <span>Edytuj nazwę</span>
    </button>
    <button v-if="selectedType == 'file'" @click="() => {menuRef?.close(); deleteFolderBox = true}">
      <Icon name="material-symbols:scan-delete-rounded"/>
      <span>Usuń plik</span>
    </button>
    <div v-if="selectedType == 'file'"/>

    <button @click="() => {menuRef?.close(); addFileBox = true}">
      <Icon name="material-symbols:file-copy-rounded"/>
      <span>Prześlij plik</span>
    </button>
    <button @click="() => {menuRef?.close(); addFolderBox = true}">
      <Icon name="material-symbols:create-new-folder-rounded"/>
      <span>Nowy folder</span>
    </button>
  </PartMiniMenu>

  <BoxFolderAdd
      :show="addFolderBox"
      @close="addFolderBox = false"
      @success="handleSuccess"
      :parent-id="parentId || undefined"/>

  <BoxDelete
      :show="deleteFolderBox"
      @close="deleteFolderBox = false"
      @success="handleSuccess"
      :type="selectedType ?? undefined"
      :id="selectedItem?.id ?? ''"
      :name="selectedItem?.name"/>

  <BoxEdit
      :show="editFolderBox"
      @close="editFolderBox = false"
      @success="handleSuccess"
      :type="selectedType ?? undefined"
      :id="selectedItem?.id ?? ''"
      :name="selectedItem?.name ?? ''"/>

  <BoxFileUpload
      :show="addFileBox"
      @close="addFileBox = false"
      @success="handleSuccess"
      :parent-id="parentId || undefined"/>
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

.default-box {
  padding-top: 1rem;
  justify-items: center;

  div {
    background: var(--box-color);
    padding: 1rem;
    border-radius: 1rem;
  }
}

.option-box {
  padding-top: 1rem;
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
      max-width: 10ch;
      text-overflow: ellipsis;
      overflow: hidden;
      white-space: nowrap;

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
  padding-top: 1rem;
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