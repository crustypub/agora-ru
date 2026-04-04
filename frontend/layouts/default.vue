<template>
  <div class="default-layout">
    <header class="layout-header">

    </header>

    <main class="layout-main">
      <slot />
    </main>

    <footer class="layout-footer">
      <p>© 2024 Мой сайт</p>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.default-layout {
  width: 100%;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  font-size: $base-font-size;

  .layout-header {
    width: 100%;
    height: 70px;
    background-color: $blue-primary;
  }

  .layout-main {
    width: 100%;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .layout-footer {
    width: 100%;
    min-height: 150px;
    background-color: $sand;
  }
}
</style>