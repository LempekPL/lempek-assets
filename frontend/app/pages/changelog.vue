<script setup lang="ts">

import HeaderBox from "~/components/HeaderBox.vue";

let changes = {
  "Version 2.0": [
    "added more file icons",
    "changed login token from one jwt to access and refresh providing better security",
    "added ability for admins to create new users",
    "added ability to drag and drop files on website to make it easier to upload files",
    "added username change",
    "added changelog",
    "added logged in device list",
    "removed ability to upload files starting with '.'",
  ],
  "Version 1.0": [
    "created folders and files",
    "created ability to log in and out",
    "created two types of item display (grid and list)",
    "created folder navigation bar",
    "added ability to create folders and files",
    "added ability to rename folders and files",
    "added ability to delete folders and files",
    "added password change",
  ]
}

</script>

<template>
  <h1>Changelog</h1>
  <HeaderBox width="min(100%, 60vh)" v-for="(features, version) in changes">
    <template #header>{{ version }}</template>
    <div class="feature-list" :id="version.replace(/ /g, '-')">
      <template v-for="feature in features">
        <p>●</p>
        <p>{{ feature }}</p>
      </template>
    </div>
  </HeaderBox>
</template>

<style lang="scss" scoped>
.feature-list {
  padding: 0 1rem;
  align-self: flex-start;
  display: grid;
  grid-template-columns: 1rem 1fr;
  gap: .5rem;

}
</style>