import type { IBaseListParams, SortOrder } from '~/models/common/filters';

/**
 * Универсальный composable для управления состоянием фильтров и сортировки в любом списке.
 * При изменении любого фильтра автоматически сбрасывает пагинацию на 1-ю страницу.
 *
 * @param initialFilters — начальные значения для специфичных полей (помимо page/limit/sort)
 *
 * @example
 * const { params, resetFilters } = useListFilters(
 *   { wiki_type: undefined, is_confirmed: undefined },
 *   { defaultSortBy: 'created_at', defaultSortOrder: 'desc' }
 * );
 */
export function useListFilters<TExtra extends Record<string, unknown>>(
  initialFilters: TExtra,
  options?: {
    defaultLimit?: number;
    defaultSortBy?: string;
    defaultSortOrder?: SortOrder;
  }
) {
  const {
    defaultLimit = 15,
    defaultSortBy = 'created_at',
    defaultSortOrder = 'desc',
  } = options ?? {};

  const page = ref(1);
  const limit = defaultLimit;

  const search = ref<string>('');
  const sort_by = ref<string>(defaultSortBy);
  const sort_order = ref<SortOrder>(defaultSortOrder);

  // Специфичные для конкретного списка фильтры (reactive-объект)
  const extra = reactive<TExtra>({ ...initialFilters });

  /** Сбрасываем страницу при изменении любого фильтра/сортировки */
  function resetPage() {
    page.value = 1;
  }

  // Следим за базовыми полями + deep-наблюдение за extra
  watch([search, sort_by, sort_order], resetPage);
  watch(extra, resetPage, { deep: true });

  /** Итоговые query-параметры для передачи в useApi */
  const params = computed<IBaseListParams & TExtra>(() => {
    const result: Record<string, unknown> = {
      page: page.value,
      limit,
      sort_by: sort_by.value,
      sort_order: sort_order.value,
    };

    // Добавляем search только если он не пустой
    if (search.value.trim()) {
      result.search = search.value.trim();
    }

    // Добавляем extra-поля, пропуская undefined/null (не засоряем query-строку)
    for (const [key, value] of Object.entries(extra)) {
      if (value !== undefined && value !== null) {
        result[key] = value;
      }
    }

    return result as IBaseListParams & TExtra;
  });

  /** Полный сброс всех фильтров к начальным значениям */
  function resetFilters() {
    page.value = 1;
    search.value = '';
    sort_by.value = defaultSortBy;
    sort_order.value = defaultSortOrder;
    Object.assign(extra, initialFilters);
  }

  return {
    page,
    limit,
    search,
    sort_by,
    sort_order,
    extra,
    params,
    resetFilters,
  };
}
