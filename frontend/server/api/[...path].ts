import { getQuery } from 'h3';

export default defineEventHandler(async (event) => {
    const config = useRuntimeConfig()
    const path = event.context.params?.path ?? ''
    const query = getQuery(event)
    const queryString = Object.keys(query).length ? '?' + new URLSearchParams(query as Record<string, string>).toString() : ''

    return proxyRequest(event, `${config.backendUrl}/api/${path}${queryString}`)
})
