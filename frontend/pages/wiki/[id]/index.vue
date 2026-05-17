<template>
  <div class="container">
    <WikiArticlePage :article-id="id"/>
  </div>
</template>

<script setup lang="ts">
import WikiArticlePage from '~/components/wiki/WikiArticlePage.vue';
import WikiTypesPage from '~/components/wiki/WikiTypesPage.vue';

const route = useRoute()
const id = route.params.id

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
