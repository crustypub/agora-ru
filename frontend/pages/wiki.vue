<template>
  <div class="container">
    <WikiTypesPage/>
  </div>
</template>

<script setup lang="ts">
import WikiTypesPage from '~/components/wiki/WikiTypesPage.vue';


useHead({
  title: 'Agora RU — Wiki',
})
</script>

<style scoped>
.container {
  min-width: 100%;
  height: 100%;
}


</style>
