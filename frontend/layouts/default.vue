<template>
  <div class="default-layout">
    <header>
      <h1>Мой сайт</h1>
    </header>
    
    <main>
      <slot /> 
    </main>
    
    <footer>
      <p>© 2024 Мой сайт</p>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.default-layout {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
}
</style>