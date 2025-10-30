<script setup lang="ts">
const props = defineProps<{
  isFolder?: boolean,
  name: string
}>();
//"material-symbols:" +
const LIST_OF_ICONS: [string[], string][] = [
  [["png", "jpg", "jpeg", "gif", "webp"], "material-symbols:imagesmode-rounded"],
  [["mov", "mp4", "webm", "mkv"], "material-symbols:video-library-rounded"],
  [["zip", "tar", "tar.gz", "rar", "7z"], "material-symbols:folder-zip-rounded"],
  [["pdf"], "material-symbols:picture-as-pdf-rounded"]
]

const iconName = computed<string>(() => {
  if (props.isFolder) {
    return "material-symbols:folder-rounded";
  }
  for (const listOfIcon of LIST_OF_ICONS) {
    for (const extName of listOfIcon[0]) {
      if (props.name.toLowerCase().endsWith("." + extName)) {
        return listOfIcon[1];
      }
    }
  }
  return "material-symbols:unknown-document-rounded";
});

</script>

<template>
  <div class="item" :title="name">
    <Icon class="icon" :name="iconName"/>
    <p>{{ name }}</p>
  </div>
</template>

<style scoped>
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

  .icon {
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
</style>