<template>
  <div class="container">
    <MainPage/>
  </div>
</template>

<script setup lang="ts">
import MainPage from '~/components/mainPage/MainPage.vue';

useHead({
  title: 'Agora RU — Главная',
})
</script>

<style scoped>
.container {
  min-width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  
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
