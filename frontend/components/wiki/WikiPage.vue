<template>
  <WikiArticleModal
    :wiki-types="wiki_types_response?.data"
    v-model="isOpenWikiCreateModal"
    :submit="onArticleCreated"
  />

  <div class="wiki-container">
    <!-- Кнопка создания -->
    <div class="wiki-create">
      <ClientOnly>
        <UButton
          icon="material-symbols:add-circle-outline-rounded"
          size="lg"
          color="primary"
          variant="soft"
          @click="openModal"
        >
          Создать статью
        </UButton>
      </ClientOnly>
    </div>

    <!-- Панель фильтров и сортировки -->
    <WikiFiltersBar
      v-model:search="search"
      v-model:sort-by="sort_by"
      v-model:sort-order="sort_order"
      v-model:wiki-type="extra.wiki_type"
      v-model:is-confirmed="extra.is_confirmed"
      :wiki-types="wiki_types_response?.data"
      @reset="resetFilters"
    />

    <!-- Скелетон на время загрузки -->
    <div class="wiki-container__content" v-if="pending">
      <WikiArticleListItemSkeleton v-for="i in skeletonCount" :key="i" />
    </div>

    <!-- Список статей -->
    <div class="wiki-container__content" v-else-if="response?.data?.length">
      <WikiArticleListItem
        v-for="article in response.data"
        :key="article.id"
        :data="article"
      />
    </div>

    <!-- Пустое состояние -->
    <div class="wiki-container__empty" v-else-if="!pending">
      <UIcon name="material-symbols:article-outline-rounded" class="wiki-container__empty-icon" />
      <p class="wiki-container__empty-text">Статьи не найдены</p>
      <UButton
        v-if="hasActiveFilters"
        variant="ghost"
        size="sm"
        @click="resetFilters"
      >
        Сбросить фильтры
      </UButton>
    </div>

    <!-- Пагинация -->
    <div class="wiki-container__pagination">
      <UPagination
        v-if="paginationValue"
        v-model:page="page"
        :items-per-page="limit"
        :total="paginationValue.total"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { IWikiResponse, IWikiTypesResponse } from '~/models/api/wiki.api';
import type { IPaginationValue } from '~/models/api/meta.api';
import WikiArticleListItem from './WikiArticleListItem.vue';
import WikiFiltersBar from './WikiFiltersBar.vue';

const {
  page,
  limit,
  search,
  sort_by,
  sort_order,
  extra,
  params,
  resetFilters,
} = useListFilters(
  {
    wiki_type:    undefined as number | undefined,
    is_confirmed: undefined as boolean | undefined,
  },
  {
    defaultSortBy: 'created_at',
    defaultSortOrder: 'desc',
    syncUrl: true,
    extraFromUrl: (q) => ({
      wiki_type:    q.wiki_type    ? Number(q.wiki_type) : undefined,
      is_confirmed: q.is_confirmed === 'true'  ? true
                  : q.is_confirmed === 'false' ? false
                  : undefined,
    }),
    extraToUrl: (extra) => {
      const out: Record<string, string> = {};
      if (extra.wiki_type    !== undefined) out.wiki_type    = String(extra.wiki_type);
      if (extra.is_confirmed !== undefined) out.is_confirmed = String(extra.is_confirmed);
      return out;
    },
  }
);


const { data: response, refresh, pending } = await useApi<IWikiResponse>('/api/wiki_articles', {
  query: params,
});

const { data: wiki_types_response } = await useApi<IWikiTypesResponse>('/api/wiki_types');

// ---- Пагинация ----
const paginationValue = computed<IPaginationValue | null>(() => {
  if (!response.value?.meta) return null;
  return {
    page: page.value,
    total: response.value.meta.total_count,
    items: limit,
  };
});

// Количество скелетонов = сколько было элементов до, иначе 5
const skeletonCount = computed(() => response.value?.data?.length || 5);


// ---- Признак активных фильтров (для пустого состояния) ----
const hasActiveFilters = computed(() =>
  !!search.value ||
  sort_by.value !== 'created_at' ||
  sort_order.value !== 'desc' ||
  extra.wiki_type !== undefined ||
  extra.is_confirmed !== undefined
);

const isOpenWikiCreateModal = ref(false);
const openModal = ():void => {
  isOpenWikiCreateModal.value = true
};

function onArticleCreated() {
  page.value = 1;
  refresh();
}
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

  &__empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 3rem 0;
    color: $text-muted;
  }

  &__empty-icon {
    font-size: 3rem;
    opacity: 0.4;
  }

  &__empty-text {
    font-size: $text-sm;
    margin: 0;
  }

  &__pagination {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}
</style>
