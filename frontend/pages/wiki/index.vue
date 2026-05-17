<template>
  <div class="container">
    <WikiPage />
  </div>
</template>

<script setup lang="ts">
import WikiPage from "~/components/wiki/WikiPage.vue";

useHead({
  title: "Agora RU — Wiki",
});
</script>

<style scoped lang="scss">
.container {
  min-width: 100%;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
}
</style>
