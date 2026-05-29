<template>
  <UCard class="filters-bar-card" :ui="{ body: 'p-3 sm:p-4' }">
    <div class="filters-bar">
      <div class="filters-bar__left">
        <UInput
          v-if="showSearch"
          v-model="searchModel"
          :placeholder="searchPlaceholder"
          size="sm"
          class="filters-bar__search"
        >
          <template #leading>
            <UIcon name="material-symbols:search-rounded" class="filters-bar__search-icon" />
          </template>
          <template #trailing>
            <UButton
              v-if="searchModel"
              icon="material-symbols:close-rounded"
              size="xs"
              variant="ghost"
              color="neutral"
              aria-label="Очистить поиск"
              @click="searchModel = ''"
            />
          </template>
        </UInput>

        <template v-if="$slots.filters">
          <slot name="filters" />
        </template>
      </div>

      <div class="filters-bar__right">
        <div class="filters-bar__sort" v-if="sortOptions.length">
          <USelect
            v-model="sortByModel"
            :items="sortOptions"
            value-key="value"
            size="sm"
            class="filters-bar__sort-select"
          />
          <UButton
            :icon="sortOrderModel === 'asc'
              ? 'material-symbols:arrow-upward-rounded'
              : 'material-symbols:arrow-downward-rounded'"
            size="sm"
            variant="ghost"
            color="neutral"
            :aria-label="sortOrderModel === 'asc' ? 'По возрастанию' : 'По убыванию'"
            @click="toggleSortOrder"
          />
        </div>

        <UButton
          v-if="hasActiveFilters"
          icon="material-symbols:filter-alt-off-outline-rounded"
          size="sm"
          variant="ghost"
          color="neutral"
          @click="emit('reset')"
        >
          Сбросить
        </UButton>
      </div>
    </div>
  </UCard>
</template>

<script setup lang="ts">
import type { ISortOption, SortOrder } from '~/models/common/filters';

interface IProps {
  sortOptions?: ISortOption[];
  showSearch?: boolean;
  searchPlaceholder?: string;
  hasActiveFilters?: boolean;
}

const {
  sortOptions = [],
  showSearch = true,
  searchPlaceholder = 'Поиск...',
  hasActiveFilters = false,
} = defineProps<IProps>();

const emit = defineEmits<{
  reset: [];
}>();

const searchModel = defineModel<string>('search', { default: '' });
const sortByModel = defineModel<string>('sortBy', { required: true });
const sortOrderModel = defineModel<SortOrder>('sortOrder', { required: true });

function toggleSortOrder() {
  sortOrderModel.value = sortOrderModel.value === 'asc' ? 'desc' : 'asc';
}
</script>

<style lang="scss" scoped>
.filters-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  flex-wrap: wrap;

  &__left {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    flex: 0 1 auto;
  }

  &__right {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
    flex-shrink: 0;
  }

  &__search {
    width: 240px;
  }

  &__search-icon {
    font-size: 1rem;
    color: var(--ui-text-muted);
  }

  &__sort {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  &__sort-select {
    min-width: 170px;
  }

  // Мобильная адаптация
  @media (max-width: 768px) {
    // Левая и правая группы — каждая на свою строку
    &__left {
      flex: 1 1 100%;
      flex-direction: column;
      align-items: stretch;
    }

    // Поиск растягивается на всю ширину
    &__search {
      width: 100%;
    }

    &__right {
      flex: 1 1 100%;
      margin-left: 0;
      justify-content: flex-end;
    }
  }
}
</style>
