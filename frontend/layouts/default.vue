<script setup lang="ts">
import Footer from '~/components/Footer.vue';



</script>

<template>
  <div class="default-layout">
    <Header />

    <main class="layout-main">
      <slot />
    </main>

    <Footer />
  </div>
</template>

<style scoped lang="scss">
.default-layout {
  width: 100%;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  font-size: $base-font-size;
  font-family: 'IBM Plex Sans', sans-serif;
  
  .layout-main {
    width: 100%;
    flex: 1;
    display: flex;
    flex-direction: column;
  }


}
</style>