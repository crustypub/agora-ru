// composables/useApi.ts
export const useApi = <T>(url: string, options: any = {}) => {
  const config = useRuntimeConfig()


  return useFetch<T>(url, {
    ...options,
    baseURL: config.public.apiBase,
    // На клиенте это заставит браузер автоматически прикреплять httpOnly куки
    credentials: 'include',
    // На сервере (SSR) это проксирует куки из входящего запроса (от браузера) в исходящий (к API)
    headers: {
      ...(options.headers || {}),
      ...useRequestHeaders(['cookie']),
    },
  })
}