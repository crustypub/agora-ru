// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    compatibilityDate: '2025-01-01',
    devtools: { enabled: true },
    css: ['~/assets/css/reset.css', '~/assets/css/font.css'],

    // Кэширование шрифта
    // nitro: {
    //     devServer: {
    //         watch: ['./server']
    //     },
    //     routeRules: {
    //         '/fonts/**': {
    //             headers: {
    //                 'Cache-Control': 'public, max-age=31536000, immutable'
    //             }
    //         }
    //     }
    // },

    experimental: {
        // Улучшает обработку динамических импортов
        payloadExtraction: true
    },


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
        port: 80,
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

    modules: ['nuxt-telegram-auth'],
    runtimeConfig: {
        // Этот токен доступен только на сервере
        TELEGRAM_TOKEN: process.env.TELEGRAM_TOKEN,

        public: {
            apiBase: process.env.NUXT_PUBLIC_API_BASE
        },
    }
})