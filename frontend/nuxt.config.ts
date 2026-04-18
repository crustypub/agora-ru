// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    compatibilityDate: '2025-01-01',
    devtools: { enabled: true },
    css: ['~/assets/css/main.css'],
    // Кэширование шрифта
    nitro: {
        devServer: {
            watch: ['./server']
        },
    },

    fonts: {
        // Используем src напрямую — модуль сгенерирует @font-face и preload в SSR HTML
        families: [
            {
                name: 'Caesar Dressing',
                src: '/fonts/CaesarDressing-Regular.ttf',
                weight: '400',
                display: 'swap',
                global: true,
            },
            {
                name: 'IBM Plex Sans',
                src: '/fonts/IBMPlexSans-Regular.ttf',
                weight: '400',
                display: 'swap',
                global: true,
            },
            {
                name: 'IBM Plex Sans',
                src: '/fonts/IBMPlexSans-SemiBold.ttf',
                weight: '600',
                display: 'swap',
                global: true,
            },
        ],
    },

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

    modules: ['nuxt-telegram-auth', '@nuxt/fonts', '@nuxt/ui', '@nuxtjs/color-mode'],

    ui: {
        theme: {
            colors: ['primary', 'secondary', 'success', 'error', 'warning'],
        },
    },
    runtimeConfig: {
        // Этот токен доступен только на сервере
        TELEGRAM_TOKEN: process.env.TELEGRAM_TOKEN,

        public: {
            apiBase: process.env.NUXT_PUBLIC_API_BASE
        },
    }
})