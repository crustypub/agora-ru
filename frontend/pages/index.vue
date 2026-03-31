<template>
  <div class="container">
    <h1>Agora RU</h1>
    <p>Фронтенд запущен и работает 🚀</p>
  </div>
</template>

<script setup lang="ts">
useHead({
  title: 'Agora RU — Главная',
})
</script>

<style scoped>
.container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  font-family: 'Inter', sans-serif;
}

h1 {
  font-size: 2.5rem;
  color: #1a1a2e;
}

p {
  font-size: 1.2rem;
  color: #555;
}
</style>
