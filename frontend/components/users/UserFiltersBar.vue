<template>
  <UiListFiltersBar
    v-model:search="search"
    :has-active-filters="hasActiveFilters"
    search-placeholder="Поиск пользователей..."
    is-simple
    @reset="emit('reset')"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import UiListFiltersBar from '~/components/ui/ListFiltersBar.vue';

const emit = defineEmits<{
  reset: [];
}>();

const search = defineModel<string>('search', { default: '' });

const hasActiveFilters = computed(() => !!search.value);
</script>

<style lang="scss" scoped>
</style>
