export const useApi = <T>(url: string | (() => string), options: any = {}) => {
  const { headers: optionHeaders, ...restOptions } = options

  return useFetch<T>(url, {
    credentials: 'include',
    headers: {
      ...useRequestHeaders(['cookie']), // пробрасываем куки при SSR
      ...(optionHeaders ?? {}),         // заголовки из options НЕ перетирают куки
    },
    ...restOptions,
  })
}

// Для императивных вызовов внутри функций (click-хэндлеры, экшены и т.д.)
// $fetch кидает FetchError при 4xx/5xx — try/catch работает нативно.
export const useApiCall = <T>(url: string, options: any = {}) => {
  return $fetch<T>(url, {
    credentials: 'include',
    ...options,
  })
}