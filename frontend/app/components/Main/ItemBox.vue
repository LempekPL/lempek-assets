<script setup lang="ts">
const props = defineProps<{
  isFolder?: boolean,
  name: string
}>();
//"material-symbols:" +
const listOfIcons: [string,string][] = [
  ["html", "a"]
]

const iconName = computed<string>(() => {
  if (props.isFolder) {
    return "material-symbols:folder-rounded";
  }
  for (const listOfIcon of listOfIcons) {
    if (props.name.endsWith("."+listOfIcon[0])) {
      return listOfIcon[1];
    }
  }
  return "material-symbols:unknown-document-rounded";
});

</script>

<template>
  <div class="item">
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