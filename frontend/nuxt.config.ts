// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    compatibilityDate: '2025-01-01',
    devtools: { enabled: true },

    css: ['~/assets/css/reset.css'],
    
    vite: {
        css: {
            preprocessorOptions: {
                scss: {
                    additionalData: `
                        @use "~/assets/scss/_variables.scss" as *;
                    `,
                },
            },
        },
    },

    devServer: {
        host: '0.0.0.0',
        port: 3000,
    },

    app: {
        head: {
            title: 'Agora RU',
            meta: [
                { charset: 'utf-8' },
                { name: 'viewport', content: 'width=device-width, initial-scale=1' },
                { name: 'description', content: 'Agora RU — Платформа' },
            ],
        },
    },
})
