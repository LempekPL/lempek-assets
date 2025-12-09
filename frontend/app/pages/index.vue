<!--suppress HtmlUnknownTarget -->
<script setup lang="ts">
import PartMiniMenu from "~/components/part/MiniMenu.vue";
import type {Folder, File, UuidName, TypedItem} from "~~/types/api";

const {copy, copying} = useClipboard();
import {useRouter} from 'vue-router';
import {useDropdown} from "~/composables/useDropdown";

const router = useRouter();
const route = useRoute();
const parentId = computed(() => route.query.parent as string | null);
const config = useRuntimeConfig();
const {isFileDrag} = useDragEvents();

const {
  show: showOrderMenu,
  toggle: orderMenuToggle,
  buttonRef: buttonOrderRef,
  dropdownRef: dropdownOrderRef
} = useDropdown();

const viewType = ref<'grid' | 'list'>('grid');
type OrderTypes = 'name_asc' | 'name_desc' | 'created_asc' | 'created_desc' | 'updated_asc' | 'updated_desc';
const orderChoice = ref<OrderTypes>('name_asc');
const VALID_ORDERS: Record<OrderTypes, string> = {
  'name_asc': "Nazwa A-Z",
  'name_desc': "Nazwa Z-A",
  'created_asc': "Aktualizacja najstarsze",
  'created_desc': "Aktualizacja najnowsze",
  'updated_asc': "Utworzenie najstarsze",
  'updated_desc': "Utworzenie najnowsze"
};
const DISPLAY_STORAGE = 'view-type';
const ORDER_STORAGE = 'order-type';
const FETCH_OPTIONS = {
  method: 'GET',
  credentials: 'include',
  headers: {'Content-Type': 'application/json'},
  watch: [parentId, orderChoice],
  lazy: true,
};

const {
  data: folders,
  pending: foldersPending,
  error,
  refresh: refreshFolders
} = await useFetch<Folder[]>(() => config.public.apiBase + `/folders?parent=${parentId.value ?? ''}&order=${orderChoice.value ?? ''}`, FETCH_OPTIONS as {});

const {
  data: files,
  pending: filesPending,
  refresh: refreshFiles
} = await useFetch<File[]>(() => config.public.apiBase + `/files?parent=${parentId.value ?? ''}&order=${orderChoice.value ?? ''}`, FETCH_OPTIONS as {});

const {
  data: folderPath,
  pending: pathPending,
} = await useFetch<UuidName[]>(() => config.public.apiBase + "/folder/path?id=" + (parentId.value ?? ''), FETCH_OPTIONS as {});

const pending = computed(() => foldersPending.value || filesPending.value || pathPending.value);

const menuRef = ref<InstanceType<typeof PartMiniMenu> | null>(null);
const selectedItem = ref<TypedItem | null>(null);
function openMenuBox(event: MouseEvent, item: Folder | File | null = null, type: 'file' | 'folder' | null = null) {
  event.preventDefault();
  menuRef.value?.open(event.clientX, event.clientY);
  if (type === 'folder') {
    item = item as Folder;
    selectedItem.value = {type: 'folder', item};
  } else {
    item = item as File;
    selectedItem.value = {type: 'file', item};
  }
}
function handleClickOutside(event: MouseEvent) {
  if (menuRef.value?.isOpen() && !menuRef.value.contains(event.target as Node)) {
    menuRef.value.close();
    selectedItem.value = null;
  }
}

onMounted(() => {
  window.addEventListener('mousedown', handleClickOutside);
  const saved = localStorage.getItem(DISPLAY_STORAGE);
  if (saved === 'grid' || saved === 'list') {
    viewType.value = saved;
  }
  const savedOrder = localStorage.getItem(ORDER_STORAGE) as OrderTypes | null;
  if (savedOrder && Object.keys(VALID_ORDERS).includes(savedOrder)) {
    orderChoice.value = savedOrder;
  }

});
onBeforeUnmount(() => {
  window.removeEventListener('mousedown', handleClickOutside);
});

type ModalType = 'addFolder' | 'uploadFile' | 'deleteItem' | 'editItem' | 'moveItem';
const modalBox = ref<ModalType | null>(null);

function enterFolder(id: string | null) {
  router.push({path: '/', query: {parent: id}});
}

async function enterFile(fileName: string) {
  const response = await $fetch<UuidName[]>(`${config.public.apiBase}/folder/path?id=${parentId.value ?? ''}`, FETCH_OPTIONS as {});
  const joinedPath = response.map(item => item.name).filter(Boolean).join("/");
  let base = new URL(config.public.filePath);
  if (joinedPath)
    base = new URL(joinedPath + '/', base);
  const fileUrl = new URL(fileName, base);
  window.location.href = fileUrl.toString();
}

function handleSuccess() {
  modalBox.value = null;
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

watch(viewType, (val) => {
  localStorage.setItem(DISPLAY_STORAGE, val);
});
watch(orderChoice, (val) => {
  if (val)
    localStorage.setItem(ORDER_STORAGE, val);
});
useHead(() => ({
  title: "AS - " + (folderPathSpliced.value.at(-1)?.name ?? "/")
}));

export type DraggedItem = TypedItem & { parentId: string | null };
const draggedItem = ref<DraggedItem | null>(null);
const droppedItem = ref<string | null | undefined>(undefined);
const droppedItemAmount = ref(0);

function dragItemEnter(e: DragEvent, folderId: string | null | undefined) {
  if (draggedItem.value && draggedItem.value.item.id !== folderId) {
    droppedItemAmount.value++;
    if (!isFileDrag(e)) {
      droppedItem.value = folderId
    }
  }
}

function dragItemLeave(e: DragEvent, folderId: string | null | undefined) {
  if (draggedItem.value && draggedItem.value.item.id !== folderId) {
    droppedItemAmount.value--;
    if (!isFileDrag(e) && droppedItemAmount.value === 0) {
      droppedItem.value = undefined
    }
  }
}

function dragItemStart(_e: DragEvent, item: Folder | File, type: 'folder' | 'file') {
  if (type === 'folder') {
    item = item as Folder;
    draggedItem.value = {type: 'folder', item, parentId: item.parent_id};
  } else {
    item = item as File;
    draggedItem.value = {type: 'file', item, parentId: item.folder_id};
  }
}

function dragItemDrop(_e: DragEvent) {
  console.log(`dragging (${draggedItem.value?.type}) ${draggedItem.value?.item.id} from ${draggedItem.value?.parentId} to (folder) ${droppedItem.value}`);
  if (droppedItem.value !== undefined && draggedItem.value?.parentId !== droppedItem.value) {
    modalBox.value = 'moveItem';
  }
  if (draggedItem.value?.parentId === droppedItem.value) {
    dragItemCancel();
  }
}

function dragItemCancel() {
  modalBox.value = null;
  droppedItem.value = undefined;
  draggedItem.value = null;
  droppedItemAmount.value = 0;
}

function dragItemFinished() {
  handleSuccess();
  droppedItem.value = undefined;
  draggedItem.value = null;
  droppedItemAmount.value = 0;
}
</script>

<template>
  <div class="option-box">
    <div>
      <div class="path-text box">
        <RouterLink
            :to="{ path: '/' }"
            :class="{ 'droppedItem': droppedItem === null }"
            @dragenter="dragItemEnter($event, null)"
            @dragleave="dragItemLeave($event, null)"
            @dragover.prevent
            @drop="dragItemDrop"
        >/
        </RouterLink>
        <template v-for="(pathItem, idx) in folderPathSpliced" :key="pathItem.id">
          <RouterLink
              :to="{ path: '/', query: { parent: pathItem.id } }"
              :class="{ 'droppedItem': droppedItem === pathItem.id }"
              @dragenter="dragItemEnter($event, pathItem.id)"
              @dragleave="dragItemLeave($event, pathItem.id)"
              @dragover.prevent
              @drop="dragItemDrop">
            <template v-if="(folderPath?.length ?? 0) > 6 && idx === 0">
              ...
            </template>
            <template v-else-if="(folderPath?.length ?? 0) > 6 && idx > 0 || (folderPath?.length ?? 0) <= 6">
              {{ pathItem.name }}
            </template>
          </RouterLink>
          <RouterLink
              :to="{ path: '/', query: { parent: pathItem.id } }"
              @dragenter="dragItemEnter($event, pathItem.id); console.log(pathItem)"
              @dragleave="dragItemLeave($event, pathItem.id)">/
          </RouterLink>
        </template>
      </div>
      <div class="sorting-option">
        <button ref="buttonOrderRef" @click="orderMenuToggle">
          <Icon class="order-icon" :class="{'rotate': showOrderMenu}" name="material-symbols:arrow-forward"/>
          <span>Sortuj po</span></button>
        <transition name="order-option" mode="out-in">
          <div ref="dropdownOrderRef" v-show="showOrderMenu" class="sorting-options">
            <MainOrderButton
                v-for="orderButton in Object.keys(VALID_ORDERS) as OrderTypes[]"
                :key="orderButton"
                @click="orderChoice = orderButton"
                :selected="orderChoice === orderButton">
              {{ VALID_ORDERS[orderButton] }}
            </MainOrderButton>
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

    <div v-else-if="pending" class="main-box">
      <div :class="viewType === 'grid' ? 'items-grid' : 'items-list'" style="overflow: hidden; height: calc(14rem * 3)">
        <MainFakeItemBox v-for="number in 1000" :key="number"/>
      </div>
    </div>

    <DragDropWrap :current="parentId" :on-success="handleSuccess"
                  v-else-if="folders?.length === 0 && files?.length === 0"
                  @contextmenu.prevent="openMenuBox($event)"
                  class="default-box">
      <div>
        <p>Brak przedmiotów w tym folderze</p>
      </div>
    </DragDropWrap>

    <DragDropWrap :current="parentId" :on-success="handleSuccess" v-else @contextmenu.prevent="openMenuBox($event)"
                  class="main-box">
      <div :class="viewType === 'grid' ? 'items-grid' : 'items-list'">
        <MainItemBox
            v-for="folder in folders" :key="folder.id"
            :class="{ 'droppedItem': droppedItem === folder.id }"
            @contextmenu.prevent.stop="openMenuBox($event, folder, 'folder')"
            @dblclick="enterFolder(folder.id)"
            :name="folder.name"
            @dragenter="dragItemEnter($event, folder.id)"
            @dragleave="dragItemLeave($event, folder.id)"
            @dragstart="dragItemStart($event, folder, 'folder')"
            @dragover.prevent
            @drop="dragItemDrop($event)"
            isFolder
        />
        <MainItemBox
            v-for="file in files" :key="file.id"
            @contextmenu.prevent.stop="openMenuBox($event, file, 'file')"
            @dblclick="enterFile(file.name)"
            @dragstart="dragItemStart($event, file, 'file')"
            :name="file.name"
            :author="file.owner_name"
        />
      </div>
    </DragDropWrap>
  </transition>

  <PartMiniMenu ref="menuRef" class="menu-part">
    <button v-if="selectedItem?.type === 'folder'"
            @click="() => {menuRef?.close(); enterFolder(selectedItem?.item?.id as string)}">
      <Icon name="material-symbols:folder-open"/>
      <span>Otwórz folder</span>
    </button>
    <button v-if="selectedItem?.type === 'folder'" @click="() => {menuRef?.close(); modalBox = 'editItem';}">
      <Icon name="material-symbols:folder-managed"/>
      <span>Edytuj nazwę</span>
    </button>
    <button v-if="selectedItem?.type === 'folder'" @click="() => {menuRef?.close(); modalBox = 'deleteItem';}">
      <Icon name="material-symbols:folder-delete-rounded"/>
      <span>Usuń folder</span>
    </button>

    <button v-if="selectedItem?.type === 'file'" @click="() => {menuRef?.close(); modalBox = 'editItem';}">
      <Icon name="material-symbols:edit-square-rounded"/>
      <span>Edytuj nazwę</span>
    </button>
    <button v-if="selectedItem?.type === 'file'" @click="() => {menuRef?.close(); modalBox = 'deleteItem';}">
      <Icon name="material-symbols:scan-delete-rounded"/>
      <span>Usuń plik</span>
    </button>
    <button :disabled="copying" v-if="selectedItem?.type === 'file' || selectedItem?.type === 'folder'"
            @click="async () => {menuRef?.close(); await copy(selectedItem?.item?.id ?? null)}">
      <Icon name="material-symbols:scan-info-rounded"/>
      <span>Kopiuj ID</span>
    </button>

    <div v-if="selectedItem?.type === 'file' || selectedItem?.type === 'folder'"/>

    <button @click="() => {menuRef?.close(); modalBox = 'uploadFile';}">
      <Icon name="material-symbols:file-copy-rounded"/>
      <span>Prześlij plik</span>
    </button>
    <button @click="() => {menuRef?.close();  modalBox = 'addFolder';}">
      <Icon name="material-symbols:create-new-folder-rounded"/>
      <span>Nowy folder</span>
    </button>
  </PartMiniMenu>

  <BoxFolderAdd
      :show="modalBox === 'addFolder'"
      @close="modalBox = null"
      @success="handleSuccess"
      :parent-id="parentId || undefined"/>

  <BoxDelete
      :show="modalBox === 'deleteItem'"
      @close="modalBox = null"
      @success="handleSuccess"
      :item="selectedItem"/>

  <BoxEdit
      :show="modalBox === 'editItem'"
      @close="modalBox = null"
      @success="handleSuccess"
      :item="selectedItem"/>

  <BoxMove
      :show="modalBox === 'moveItem'"
      @close="dragItemCancel"
      @success="dragItemFinished"
      :dragged="draggedItem"
      :new-parent="droppedItem"/>

  <BoxFileUpload
      :show="modalBox === 'uploadFile'"
      @close="modalBox = null"
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
      height: 100%;
      align-content: center;

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
      font-size: 1rem;
      z-index: 9;
      border: #fff7 solid 1px;
      height: 100%;
      align-items: center;

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
      top: calc(100%);
      left: 0;
      width: 100%;
      box-shadow: #000f 0 .5rem 1rem;
      border-radius: 1rem;
      z-index: 8;
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

.droppedItem {
  border: 4px solid var(--accent-color) !important;
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