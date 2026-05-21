import type { IBaseListParams, SortOrder } from '~/models/common/filters';

export interface IListFiltersUrlOptions<TExtra extends Record<string, unknown>> {
  /** Разбирает query-параметры URL в объект extra-фильтров */
  extraFromUrl?: (query: Record<string, string>) => Partial<TExtra>;
  /** Сериализует extra-фильтры обратно в query-параметры */
  extraToUrl?: (extra: TExtra) => Record<string, string>;
}

export interface IListFiltersOptions<TExtra extends Record<string, unknown>>
  extends IListFiltersUrlOptions<TExtra> {
  defaultLimit?: number;
  defaultSortBy?: string;
  defaultSortOrder?: SortOrder;
  /** Синхронизировать фильтры с URL query-параметрами */
  syncUrl?: boolean;
  /** Задержка debounce для поиска (мс). По умолчанию 350 */
  searchDebounce?: number;
}

/**
 * Универсальный composable для управления фильтрами, сортировкой и пагинацией.
 *
 * Особенности:
 * - Debounce поиска — не спамит API на каждый символ
 * - Опциональная синхронизация с URL — поделиться ссылкой, работает кнопка «Назад»
 * - При изменении фильтров автоматически сбрасывает на 1-ю страницу
 */
export function useListFilters<TExtra extends Record<string, unknown>>(
  initialFilters: TExtra,
  options?: IListFiltersOptions<TExtra>
) {
  const {
    defaultLimit = 15,
    defaultSortBy = 'created_at',
    defaultSortOrder = 'desc' as SortOrder,
    syncUrl = false,
    searchDebounce = 350,
    extraFromUrl,
    extraToUrl,
  } = options ?? {};

  // --- Основное состояние ---
  const page = ref(1);
  const limit = defaultLimit;
  const search = ref('');           // raw — биндится на UInput
  const debouncedSearch = ref(''); // delayed — уходит в params и URL
  const sort_by = ref<string>(defaultSortBy);
  const sort_order = ref<SortOrder>(defaultSortOrder);
  const extra = reactive<TExtra>({ ...initialFilters });

  // --- Debounce поиска ---
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  watch(search, (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      debouncedSearch.value = val.trim();
    }, searchDebounce);
  });

  onUnmounted(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  // --- Сброс пагинации при изменении фильтров ---
  function resetPage() {
    page.value = 1;
  }

  // Сброс страницы привязан к debounced-значению, а не к raw search
  watch([debouncedSearch, sort_by, sort_order], resetPage);
  watch(extra, resetPage, { deep: true });

  // --- Итоговые query-параметры для useApi ---
  const params = computed<IBaseListParams & TExtra>(() => {
    const result: Record<string, unknown> = {
      page: page.value,
      limit,
      sort_by: sort_by.value,
      sort_order: sort_order.value,
    };

    if (debouncedSearch.value) {
      result.search = debouncedSearch.value;
    }

    for (const [key, value] of Object.entries(extra)) {
      if (value !== undefined && value !== null) {
        result[key] = value;
      }
    }

    return result as IBaseListParams & TExtra;
  });

  // --- Сброс всего ---
  function resetFilters() {
    page.value = 1;
    search.value = '';
    debouncedSearch.value = '';
    sort_by.value = defaultSortBy;
    sort_order.value = defaultSortOrder;
    Object.assign(extra, initialFilters);
  }

  // --- URL sync ---
  if (syncUrl) {
    const route = useRoute();
    const router = useRouter();

    // Инициализация из URL при первом рендере
    const q = route.query as Record<string, string>;

    if (q.search)     { search.value = q.search; debouncedSearch.value = q.search; }
    if (q.sort_by)    sort_by.value = q.sort_by;
    if (q.sort_order) sort_order.value = q.sort_order as SortOrder;
    if (q.page)       page.value = Math.max(1, Number(q.page));

    if (extraFromUrl) {
      Object.assign(extra, extraFromUrl(q));
    }

    // Запись в URL при изменении params
    // Чистим дефолтные значения — URL остаётся коротким
    watch(params, () => {
      const query: Record<string, string> = {};

      if (debouncedSearch.value)          query.search = debouncedSearch.value;
      if (sort_by.value !== defaultSortBy)         query.sort_by = sort_by.value;
      if (sort_order.value !== defaultSortOrder)   query.sort_order = sort_order.value;
      if (page.value > 1)                          query.page = String(page.value);

      if (extraToUrl) {
        Object.assign(query, extraToUrl(extra as TExtra));
      }

      router.replace({ query });
    }, { deep: true });
  }

  return {
    page,
    limit,
    search,        // для v-model на UInput
    sort_by,
    sort_order,
    extra,
    params,
    resetFilters,
  };
}
