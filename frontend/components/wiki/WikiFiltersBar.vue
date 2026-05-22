<template>
  <UiListFiltersBar
    v-model:search="search"
    v-model:sort-by="sort_by"
    v-model:sort-order="sort_order"
    :sort-options="SORT_OPTIONS"
    :has-active-filters="hasActiveFilters"
    search-placeholder="Поиск..."
    @reset="emit('reset')"
  >
    <template #filters>
      <USelect
        v-model="wikiTypeString"
        :items="wikiTypeOptions"
        value-key="value"
        size="sm"
        class="wiki-filters__type-select"
      />

      <USelect
        v-model="isConfirmedString"
        :items="CONFIRMED_OPTIONS"
        value-key="value"
        size="sm"
        class="wiki-filters__status-select"
      />
    </template>
  </UiListFiltersBar>
</template>

<script setup lang="ts">
import type { SortOrder, ISortOption } from '~/models/common/filters';
import type { IWikiType } from '~/models/entities/wiki.entities';

interface IProps {
  wikiTypes?: IWikiType[];
}

const { wikiTypes = [] } = defineProps<IProps>();

const emit = defineEmits<{
  reset: [];
}>();

const search = defineModel<string>('search', { default: '' });
const sort_by = defineModel<string>('sortBy', { required: true });
const sort_order = defineModel<SortOrder>('sortOrder', { required: true });
const wikiTypeValue = defineModel<number | undefined>('wikiType', { default: undefined });
const isConfirmedValue = defineModel<boolean | undefined>('isConfirmed', { default: undefined });

const SORT_OPTIONS: ISortOption[] = [
  { label: 'По дате создания',   value: 'created_at'  },
  { label: 'По дате обновления', value: 'updated_at'  },
  { label: 'По звёздам',         value: 'stars_count' },
];

// ---- Wiki type: USelect работает со строками — конвертируем id в string ----
const wikiTypeOptions = computed(() => [
  { label: 'Все типы', value: 'all' },
  ...wikiTypes.map(t => ({ label: t.title, value: String(t.id) })),
]);

const wikiTypeString = computed({
  get: () => wikiTypeValue.value !== undefined ? String(wikiTypeValue.value) : 'all',
  set: (val: string) => {
    wikiTypeValue.value = val === 'all' ? undefined : Number(val);
  },
});

const CONFIRMED_OPTIONS = [
  { label: 'Все статьи',             value: 'all'   },
  { label: 'Подтверждённые',         value: 'true'  },
  { label: 'Неподтверждённые',       value: 'false' },
];

const isConfirmedString = computed({
  get: () => {
    if (isConfirmedValue.value === true)  return 'true';
    if (isConfirmedValue.value === false) return 'false';
    return 'all';
  },
  set: (val: string) => {
    isConfirmedValue.value = val === 'all' ? undefined : val === 'true';
  },
});

// ---- Признак активных фильтров ----
const hasActiveFilters = computed(() =>
  !!search.value ||
  sort_by.value !== 'created_at' ||
  sort_order.value !== 'desc' ||
  wikiTypeValue.value !== undefined ||
  isConfirmedValue.value !== undefined
);
</script>

<style lang="scss" scoped>
.wiki-filters {
  &__type-select {
    min-width: 130px;
  }

  &__status-select {
    min-width: 170px;
  }
}
</style>
