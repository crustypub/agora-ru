// baseURL не нужен: запросы /api/** перехватывает серверный прокси Nuxt (server/api/[...path].ts),
// который сам пересылает их на backend. Работает одинаково при SSR и на клиенте.
export const useApi = <T>(url: string, options: any = {}) => {
  return useFetch<T>(url, {
    key: url, // Добавляем уникальный ключ, иначе все вызовы получат одинаковый авто-сгенерированный ключ Nuxt
    credentials: 'include',
    headers: {
      ...useRequestHeaders(['cookie']),
    },
    ...options,
  })
}