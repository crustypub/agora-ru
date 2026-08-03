<template>
  <div class="userlist-container">
    <UserFiltersBar v-model:search="search" @reset="resetFilters" />

    <!-- Скелетон на время загрузки -->
    <div class="userlist-container__content" v-if="pending">
      <!-- <WikiArticleListItemSkeleton v-for="i in skeletonCount" :key="i" /> -->
    </div>

    <!-- Список статей -->
    <div class="userlist-container__content" v-else-if="response?.data?.length">
      <UserListItem v-for="user in response.data" :key="user.id" :data="user" />
    </div>

    <div class="userlist-container__pagination">
      <UPagination v-if="paginationValue" v-model:page="page" :items-per-page="limit" :total="paginationValue.total" />
    </div>

  </div>
</template>
<script setup lang="ts">
import type { IUsersResponse } from '~/models/api/user.api.js';
import UserFiltersBar from './UserFiltersBar.vue';
import type { IPaginationValue } from '~/models/api/meta.api.js';
import UserListItem from './UserListItem.vue';

const {
  page,
  limit,
  search,
  params,
  resetFilters,
} = useListFilters(
  {},
  {
    syncUrl: true,
    searchKey: 'search_value',
  }
);

const { data: response, refresh, pending } = await useApi<IUsersResponse>('/api/users', {
  query: params,
});

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
.userlist-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  row-gap: 1rem;

  &__content {
    display: grid;
    flex:1;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  &__pagination {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}
</style>