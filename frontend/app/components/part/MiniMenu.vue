<script setup lang="ts">

const menuBoxRef = ref<HTMLElement | null>(null);
const menuBoxVisible = ref(false);
const menuBoxReady = ref(false);
const menuBoxX = ref(0);
const menuBoxY = ref(0);

function getScrollBarWidth () {
  var inner = document.createElement('p');
  inner.style.width = "100%";
  inner.style.height = "200px";

  var outer = document.createElement('div');
  outer.style.position = "absolute";
  outer.style.top = "0px";
  outer.style.left = "0px";
  outer.style.visibility = "hidden";
  outer.style.width = "200px";
  outer.style.height = "150px";
  outer.style.overflow = "hidden";
  outer.appendChild (inner);

  document.body.appendChild (outer);
  var w1 = inner.offsetWidth;
  outer.style.overflow = 'scroll';
  var w2 = inner.offsetWidth;
  if (w1 == w2) w2 = outer.clientWidth;

  document.body.removeChild (outer);

  return (w1 - w2);
}

function open(x: number, y: number) {
  menuBoxVisible.value = true;
  menuBoxReady.value = false;
  menuBoxX.value = x;
  menuBoxY.value = y;

  setTimeout(() => {
    const box = menuBoxRef.value;
    if (!box) return;
    const boxW = box.offsetWidth;
    const boxH = box.offsetHeight;
    const winW = window.innerWidth;
    const winH = window.innerHeight;
    let newX = x, newY = y;
    const scrollBarWidth = getScrollBarWidth();
    if (x + boxW + 16 + scrollBarWidth > winW) newX = winW - boxW - 16 - scrollBarWidth;
    if (y + boxH + 16 > winH) newY = y - boxH;
    menuBoxX.value = Math.max(newX, 0);
    menuBoxY.value = Math.max(newY, 0);
    menuBoxReady.value = true;
  }, 0);
}

function close() {
  menuBoxVisible.value = false;
  menuBoxReady.value = false;
  menuBoxX.value = 0;
  menuBoxY.value = 0;
}

function isOpen(): boolean {
  return menuBoxVisible.value;
}

function contains(target: Node): boolean {
  return menuBoxRef.value?.contains(target) ?? false;
}

defineExpose({
  open,
  close,
  isOpen,
  contains,
});
</script>

<template>
  <div v-if="menuBoxVisible" class="menu-box" ref="menuBoxRef" :style="{ visibility: menuBoxReady ? 'visible' : 'hidden' }">
    <slot/>
  </div>
</template>

<style scoped>
.menu-box {
  position: fixed;
  top: calc(v-bind(menuBoxY) * 1px );
  left: calc(v-bind(menuBoxX) * 1px );
  display: flex;
  flex-direction: column;
}
</style>