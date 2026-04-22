// Универсальный прокси: перехватывает все запросы /api/**
// и пересылает их на backend, добавляя куки из браузера.
// Адрес backend знает только сервер (BACKEND_URL) — в браузер не утекает.
export default defineEventHandler(async (event) => {
    const config = useRuntimeConfig()
    const path = event.context.params?.path ?? ''

    const cookie = getHeader(event, 'cookie')
    const method = event.method
    const body = ['GET', 'HEAD'].includes(method) ? undefined : await readBody(event)
    const query = getQuery(event)

    return await $fetch(`${config.backendUrl}/api/${path}`, {
        method,
        body,
        query,
        headers: {
            'Content-Type': 'application/json',
            ...(cookie ? { cookie } : {}),
        },
    })
})
