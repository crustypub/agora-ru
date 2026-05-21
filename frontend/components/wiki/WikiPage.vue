<template>
  <WikiArticleCreateModal :wiki-types="wiki_types_response?.data" v-model="isOpenWikiCreateModal" :submit="wikiArticleSubmit"/>
  <div class="wiki-container">
    <div class="wiki-create">
      <ClientOnly>
        <UButton icon="material-symbols:add-circle-outline-rounded" size="lg" color="primary" variant="soft"
          @click="openModal()">Создать статью</UButton>
      </ClientOnly>
    </div>
    <div class="wiki-container__content" v-if="!!response?.data">
      <WikiArticleListItem v-for="value in response?.data" :data="value" />
    </div>

    <div class="wiki-container__pagination">
      <UPagination v-if="!!paginationValue" v-model:page="page" :items-per-page="limit"
        :total="paginationValue.total" />

    </div>

  </div>
</template>

<script setup lang="ts">
import type { IPaginationValue } from '~/models/api/meta.api';
import type { IWikiResponse } from '~/models/api/wiki.api';
import WikiArticleListItem from './WikiArticleListItem.vue';



interface IProps { }

const { } = defineProps<IProps>();

const limit = 15;
const page = ref(1);

const { data: response, refresh } = await useApi<IWikiResponse>("/api/wiki_articles", {
  query: computed(() => ({
    page: page.value,
    limit,
  }))
});
const { data: wiki_types_response } = await useApi<IWikiResponse>("/api/wiki_types");

const isOpenWikiCreateModal = ref(false);

const openModal = () => (isOpenWikiCreateModal.value = true);

const wikiArticleSubmit = function() {
  page.value = 1;
  refresh();
}

const paginationValue = computed<IPaginationValue | null>(() => {
  if (!response.value?.meta) return null;
  return {
    page: page.value,
    total: response.value.meta.total_count,
    items: limit,
  };
});

</script>

<style lang="scss" scoped>
.wiki-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  row-gap: 1rem;

  .wiki-create {
    width: 100%;
    min-height: 42px;
    height: auto;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    margin-top: 0.5rem;
  }


  &__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    row-gap: 1rem;
  }

  &__pagination {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}
</style>
