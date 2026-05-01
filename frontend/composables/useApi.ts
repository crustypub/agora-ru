// baseURL не нужен: запросы /api/** перехватывает серверный прокси Nuxt (server/api/[...path].ts),
// который сам пересылает их на backend. Работает одинаково при SSR и на клиенте.
export const useApi = <T>(url: string, options: any = {}) => {
  const { headers: optionHeaders, ...restOptions } = options

  return useFetch<T>(url, {
    // Уникальный ключ с учётом параметров — предотвращает коллизии кеша Nuxt
    key: `${url}_${JSON.stringify(restOptions)}`,
    credentials: 'include',
    headers: {
      ...useRequestHeaders(['cookie']), // пробрасываем куки при SSR
      ...(optionHeaders ?? {}),         // заголовки из options НЕ перетирают куки
    },
    ...restOptions,
  })
}