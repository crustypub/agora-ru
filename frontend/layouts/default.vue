<template>
  <div class="default-layout">
    <Header />

    <main class="layout-main">
      <slot />
    </main>

    <MobileNav />
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped lang="scss">
.default-layout {
  width: 100%;
  height: 100dvh;
  display: flex;
  flex-direction: column;
  font-size: $base-font-size;
  font-family: 'IBM Plex Sans', sans-serif;
  overflow: hidden;
  
  .layout-main {
    width: 100%;
    flex: 1 1 auto;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: .5rem 1rem;
    max-width: 1150px;
    margin: 0 auto;
  }
}
</style>