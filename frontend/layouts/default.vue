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
  min-height: 100dvh;
  display: flex;
  flex-direction: column;
  font-size: $base-font-size;
  font-family: 'IBM Plex Sans', sans-serif;
  
  .layout-main {
    width: 100%;
    height: 100%;
    flex: 1 0 auto;
    display: flex;
    flex-direction: column;
    padding: .5rem 1rem;
    max-width: 1150px;
    margin: 0 auto;
  }


}
</style>