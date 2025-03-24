<script setup>
import {useAuthStore} from "~/stores/auth.js";

const auth = useAuthStore()
const config = useRuntimeConfig();
const { data: folders, pending, error } = await useFetch(config.public.apiBase+"/folders", {
  method: 'GET',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  }
});

// Optional: Transform data into hierarchical structure
const folderTree = computed(() => {
  const map = {};
  const roots = [];

  // First pass: create map of all folders
  folders.value.forEach(folder => {
    map[folder.path] = { ...folder, children: [] };
  });

  // Second pass: build hierarchy
  folders.value.forEach(folder => {
    const pathParts = folder.path.split('/');
    if (pathParts.length > 1) {
      const parentPath = pathParts.slice(0, -1).join('/');
      if (map[parentPath]) {
        map[parentPath].children.push(map[folder.path]);
      }
    } else {
      roots.push(map[folder.path]);
    }
  });

  return roots;
});
</script>

<template>
  <div>
    <div v-if="pending">Loading folders...</div>

    <div v-if="error" class="error">
      Error loading folders: {{ error.message }}
    </div>

    <div v-else>
      <h2>Folders</h2>
      <ul>
        <li v-for="folder in folders" :key="folder.id">
          {{ folder.name }} ({{ folder.path }})
          <span class="date">{{ new Date(folder.created_at).toLocaleDateString() }}</span>
        </li>
      </ul>

      <div class="tree">
        <FolderTree :folders="folderTree" />
      </div>
    </div>
  </div>
</template>

<!-- Recursive folder tree component -->
<!--<script setup>-->
<!--const props = defineProps({-->
<!--  folders: Array-->
<!--});-->

<!--const emit = defineEmits(['select']);-->
<!--</script>-->

<!--<template>-->
<!--  <ul>-->
<!--    <li v-for="folder in folders" :key="folder.id">-->
<!--      <div @click="emit('select', folder)">-->
<!--        {{ folder.name }}-->
<!--        <span v-if="folder.children.length">▸</span>-->
<!--      </div>-->
<!--      <FolderTree-->
<!--          v-if="folder.children.length"-->
<!--          :folders="folder.children"-->
<!--          @select="emit('select', $event)"-->
<!--      />-->
<!--    </li>-->
<!--  </ul>-->
<!--</template>-->

<style>
.tree ul {
  list-style: none;
  padding-left: 20px;
}

.tree li {
  margin: 5px 0;
  cursor: pointer;
}

.tree li div:hover {
  background-color: #f0f0f0;
}

.date {
  color: #666;
  font-size: 0.9em;
  margin-left: 10px;
}

.error {
  color: #dc3545;
}
</style>

<!--import {useAuthStore} from "~/stores/auth.js";-->

<!--const auth = useAuthStore()-->



<!--</script>-->

<!--<template>-->


<!--&lt;!&ndash;  <div>&ndash;&gt;-->
<!--&lt;!&ndash;    <h1>Dashboard</h1>&ndash;&gt;-->
<!--&lt;!&ndash;    <div v-if="auth.user">&ndash;&gt;-->
<!--&lt;!&ndash;      <p>Login: {{ auth.user.login }}</p>&ndash;&gt;-->
<!--&lt;!&ndash;      <p>Allowed: {{ auth.user.allow_upload }}</p>&ndash;&gt;-->
<!--&lt;!&ndash;    </div>&ndash;&gt;-->
<!--&lt;!&ndash;  </div>&ndash;&gt;-->
<!--</template>-->

<!--<style scoped>-->

<!--</style>-->