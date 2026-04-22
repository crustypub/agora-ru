// baseURL не нужен: запросы /api/** перехватывает серверный прокси Nuxt (server/api/[...path].ts),
// который сам пересылает их на backend. Работает одинаково при SSR и на клиенте.
export const useApi = <T>(url: string, options: any = {}) => {
  return useFetch<T>(url, {
    credentials: 'include',
    headers: {
      ...useRequestHeaders(['cookie']),
    },
    ...options,
  })
}