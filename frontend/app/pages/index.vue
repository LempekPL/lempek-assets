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
const viewType = ref<'grid' | 'list'>('grid');
type OrderTypes = 'name_asc' | 'name_desc' | 'created_asc' | 'created_desc' | 'updated_asc' | 'updated_desc';
const orderChoice = ref<OrderTypes>(orderInfo.value as OrderTypes);
const showOrderMenu = ref<boolean>(false);
const orderMenuRef = ref<HTMLElement | null>(null);
const openOrderMenuRef = ref<HTMLElement | null>(null);


const VALID_ORDERS: OrderTypes[] = [
  'name_asc', 'name_desc',
  'created_asc', 'created_desc',
  'updated_asc', 'updated_desc'
];
const DISPLAY_STORAGE = 'view-type';
const ORDER_STORAGE = 'order-type';
const FETCH_OPTIONS = {
  method: 'GET',
  credentials: 'include',
  headers: {'Content-Type': 'application/json'},
  watch: [parentId, orderChoice],
};

const {
  data: folders,
  pending,
  error,
  refresh: refreshFolders
} = await useFetch<Folder[]>(() => config.public.apiBase + `/folders?parent=${parentId.value ?? ''}&order=${orderChoice.value ?? ''}`, FETCH_OPTIONS as {});

const {
  data: files,
  refresh: refreshFiles
} = await useFetch<File[]>(() => config.public.apiBase + `/files?parent=${parentId.value ?? ''}&order=${orderChoice.value ?? ''}`, FETCH_OPTIONS as {});

const {
  data: folderPath,
} = await useFetch<UuidName[]>(() => config.public.apiBase + "/folder/path?id=" + (parentId.value ?? ''), FETCH_OPTIONS as {});

const menuRef = ref<InstanceType<typeof PartMiniMenu> | null>(null);
const selectedItem = ref<Folder | File | null>(null);
const selectedType = ref<'folder' | 'file' | null>(null);

function openMenuBox(event: MouseEvent, item: Folder | File | null = null, type: 'file' | 'folder' | null = null) {
  event.preventDefault();
  menuRef.value?.open(event.clientX, event.clientY);
  selectedItem.value = item;
  selectedType.value = type;
}

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value?.isOpen() && !menuRef.value.contains(event.target as Node)) {
    menuRef.value.close();
    selectedItem.value = null;
    selectedType.value = null;
  }
  if (!openOrderMenuRef.value?.contains(event.target as Node) && !orderMenuRef.value?.contains(event.target as Node)) {
    showOrderMenu.value = false;
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside);
  const saved = localStorage.getItem(DISPLAY_STORAGE);
  if (saved === 'grid' || saved === 'list') {
    viewType.value = saved;
  }
  const savedOrder = localStorage.getItem(ORDER_STORAGE) as OrderTypes | null;
  if (savedOrder && VALID_ORDERS.includes(savedOrder)) {
    orderChoice.value = savedOrder;
  }
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

async function enterFile(fileName: string) {
  const response = await $fetch<UuidName[]>(`${config.public.apiBase}/folder/path?id=${parentId.value ?? ''}`, FETCH_OPTIONS as {});
  const joinedPath = response.map(item => item.name).filter(Boolean).join("/");
  window.location.href = config.public.filePath + joinedPath + "/" + fileName;
}

function handleSuccess() {
  addFolderBox.value = false;
  deleteFolderBox.value = false;
  editFolderBox.value = false;
  addFileBox.value = false;
  refreshFolders();
  refreshFiles();
}

const folderPathSpliced = computed(() => {
  if (folderPath.value && folderPath.value.length > 7) {
    return folderPath.value.slice(folderPath.value.length - 7);
  } else {
    return folderPath.value || [];
  }
})

const head_title = computed(() => {
  return "AS - " + (folderPathSpliced.value[0]?.name ?? "/");
})

watch(viewType, (val) => {
  localStorage.setItem(ORDER_STORAGE, val);
});
watch(orderChoice, (val) => {
  localStorage.setItem(ORDER_STORAGE, val);
});

useHead({
  title: head_title
})
</script>

<template>
  <div class="option-box">
    <div>
      <div class="path-text box">
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
      <div class="sorting-option">
        <button ref="openOrderMenuRef" @click="showOrderMenu = !showOrderMenu">
          <Icon class="order-icon" :class="{'rotate': showOrderMenu}" name="material-symbols:arrow-forward"/>
          <span>Sortuj po</span></button>
        <transition name="order-option" mode="out-in">
          <div ref="orderMenuRef" v-show="showOrderMenu" class="sorting-options">
            <button @click="orderChoice = 'name_asc'"
                    :style="{fontWeight: orderChoice === 'name_asc' ? 'bold' : 'normal'}">Nazwa A-Z
            </button>
            <button @click="orderChoice = 'name_desc'"
                    :style="{fontWeight: orderChoice === 'name_desc' ? 'bold' : 'normal'}">Nazwa Z-A
            </button>
            <button @click="orderChoice = 'updated_asc'"
                    :style="{fontWeight: orderChoice === 'updated_asc' ? 'bold' : 'normal'}">Aktualizacja najstarsze
            </button>
            <button @click="orderChoice = 'updated_desc'"
                    :style="{fontWeight: orderChoice === 'updated_desc' ? 'bold' : 'normal'}">Aktualizacja najnowsze
            </button>
            <button @click="orderChoice = 'created_asc'"
                    :style="{fontWeight: orderChoice === 'created_asc' ? 'bold' : 'normal'}">Utworzenie najstarsze
            </button>
            <button @click="orderChoice = 'created_desc'"
                    :style="{fontWeight: orderChoice === 'created_desc' ? 'bold' : 'normal'}">Utworzenie najnowsze
            </button>
          </div>
        </transition>
      </div>
      <div class="list-grid-option" @click="viewType = viewType === 'grid' ? 'list' : 'grid'">
        <Icon v-show="viewType === 'grid'" name="material-symbols:grid-on-outline"/>
        <Icon v-show="viewType === 'list'" name="material-symbols:lists-rounded"/>
      </div>
    </div>
  </div>

  <transition name="item" mode="out-in">
    <div v-if="error" class="error">
      Error: {{ error.message }}
    </div>

    <div v-else-if="pending" class="default-box">
      <div>
        <p>Ładowanie folderów...</p>
      </div>
    </div>

    <div v-else-if="folders?.length === 0 && files?.length === 0"
         @contextmenu.prevent="openMenuBox($event)"
         class="default-box">
      <div>
        <p>Brak przedmiotów w tym folderze</p>
      </div>
    </div>

    <div v-else @contextmenu.prevent="openMenuBox($event)" class="main-box">
      <div :class="viewType === 'grid' ? 'items-grid' : 'items-list'">
        <MainItemBox
            v-for="folder in folders" :key="folder.id"
            @contextmenu.prevent.stop="openMenuBox($event, folder, 'folder')"
            @dblclick="enterFolder(folder.id)"
            :name="folder.name"
            isFolder
        />
        <MainItemBox
            v-for="file in files" :key="file.id"
            @contextmenu.prevent.stop="openMenuBox($event, file, 'file')"
            @dblclick="enterFile(file.id)"
            :name="file.name"
        />
      </div>
    </div>
  </transition>

  <PartMiniMenu ref="menuRef" class="menu-part">
    <button v-if="selectedType == 'folder'" @click="() => {menuRef?.close(); enterFolder(selectedItem?.id as string)}">
      <Icon name="material-symbols:folder-open"/>
      <span>Otwórz folder</span>
    </button>
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
.default-box {
  width: 100%;
  padding-top: 1rem;
  justify-items: center;
  flex-grow: 1; // needed for context menu

  div {
    background: var(--box-color);
    padding: 1rem;
    border-radius: 1rem;
  }
}

.option-box {
  width: 100%;
  justify-items: center;
  display: grid;

  > div {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    justify-content: space-between;
    width: 90%;

    & > .box {
      line-height: 1;
      background: var(--box-color);
      border-radius: .5rem;
      padding: 1rem;
    }
  }

  .path-text {
    width: 100%;

    & > * {
      display: inline-block;
      padding: 0 .5rem;
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

  .sorting-option {
    position: relative;

    & > button {
      position: relative;
      display: flex;
      flex-direction: row;
      gap: 0.5rem;
      width: 10rem;
      cursor: pointer;
      background: var(--box-color);
      border-radius: .5rem;
      padding: 1rem;
      border: none;
      font-size: 1rem;
      z-index: 9;

      .order-icon {
        transition: 150ms;
        transform: rotateZ(0deg);
      }

      .rotate {
        transform: rotateZ(90deg);
      }

      &:hover {
        filter: brightness(75%);
      }
    }

    .sorting-options {
      display: flex;
      flex-direction: column;
      position: absolute;
      top: calc(100% + 1rem);
      left: 0;
      width: 100%;
      box-shadow: #000 0 .5rem 1rem;
      border-radius: 1rem;
      z-index: 8;
      border: #fff7 solid 1px;

      > * {
        background: var(--box-color);
        padding: 1rem;
        cursor: pointer;
        border: none;
        font-size: 1rem;
        color: var(--text-color);

        .selected {
          filter: brightness(50%);
        }

        &:first-child {
          border-radius: 1rem 1rem 0 0;
        }

        &:last-child {
          border-radius: 0 0 1rem 1rem;
        }

        &:hover {
          filter: brightness(75%);
        }

        &:active {
          filter: brightness(50%);
        }
      }
    }
  }

  .list-grid-option {
    background: var(--button-color);
    cursor: pointer;
    aspect-ratio: 1/1;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    border-radius: 1rem;

    & > * {
      vertical-align: middle;
      scale: 2;
    }

    &:hover {
      filter: brightness(75%);
    }

    &:active {
      filter: brightness(75%);
      transform: scale(0.95);
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
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
  gap: 1rem;
}

.items-list {
  user-select: none;
  display: flex;
  flex-direction: column;
  gap: 1rem;

  .item {
    display: flex;
    flex-direction: row;
    background: var(--box-color);
    gap: 1rem;
    padding: 1rem;
    border-radius: 1rem;
    cursor: pointer;
    align-items: center;


    .icon {
      padding: .75rem;
    }

    p {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 100%;
      line-height: 1;
      vertical-align: middle;

    }
  }
}

.main-box {
  position: relative;
  width: 100%;
  flex: 1;
  padding: 1rem;
}

.error {
  color: #dc3545;
}

.dragged {

}

.item-enter-active, .item-leave-active {
  transition: opacity 0.25s;
  display: inline;
}

.item-enter-from, .item-leave-to {
  opacity: 0;
  display: inline;
}

.item-enter-to, .item-leave-from {
  opacity: 1;
  display: block;
}

.order-option-enter-active, .order-option-leave-active {
  transition: 300ms ease-out;
}

.order-option-enter-from, .order-option-leave-to {
  transform: translateY(-2rem);
  opacity: 0;
}

.order-option-enter-to, .order-option-leave-from {
  transform: translateY(0);
  opacity: 1;
}
</style>