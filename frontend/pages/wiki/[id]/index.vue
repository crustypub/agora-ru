<template>
  <div class="page-container">
    <WikiArticlePage :article-id="id"/>
  </div>
</template>

<script setup lang="ts">
import WikiArticlePage from '~/components/wiki/WikiArticlePage.vue';

const route = useRoute()
const id = route.params.id

useHead({
  title: 'Agora RU — Wiki',
})
</script>

