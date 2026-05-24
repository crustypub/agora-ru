/** Направление сортировки */
export type SortOrder = 'asc' | 'desc';

/** Базовый интерфейс одного варианта сортировки (для USelectMenu) */
export interface ISortOption<TValue extends string = string> {
  label: string;
  value: TValue;
}

/** Общий тип query-параметров запроса для любого списка */
export interface IBaseListParams {
  page: number;
  limit: number;
  sort_by?: string;
  sort_order?: SortOrder;
  search?: string;
  [key: string]: unknown;
}
