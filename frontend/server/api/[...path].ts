
export default defineEventHandler(async (event) => {
    const config = useRuntimeConfig()
    const path = event.context.params?.path ?? ''

    return proxyRequest(event, `${config.backendUrl}/api/${path}`)
})
