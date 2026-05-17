<template>
  <div class="wiki-container">
    <div class="wiki-container__content" v-if="!!response?.data">
      <WikiArticleListItem v-for="value in response?.data" :data="value"/>
    </div>

    <div class="wiki-container__pagination">
      <UPagination v-if="!!paginationValue" v-model:page="paginationValue.page" :items-per-page="limit"
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

const { data: response } = await useApi<IWikiResponse>("/api/wiki_articles");

const getPaginationValue = function (data: IWikiResponse | undefined) {
  if (data) {
    return {
      page: 1,
      total: data?.meta?.total_count,
      items: limit
    }
  }
  return null
}



const paginationValue = ref<IPaginationValue | null>(response ? getPaginationValue(response.value) : null)

</script>

<style lang="scss" scoped>
.wiki-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;

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
