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

    fonts: {
        families: [
            {
                name: 'Caesar Dressing',
                provider: 'local',
                src: '/fonts/CaesarDressing-Regular.ttf',
                display: 'swap',
                weight: 400,
                style: 'normal',
                fallbacks: ['Arial']
            },
            {
                name: 'IBM Plex Sans',
                provider: 'local',
                src: '/fonts/IBMPlexSans-Regular.ttf',
                display: 'swap',
                weight: 400,
                style: 'normal',
                fallbacks: ['Arial']
            },
            {
                name: 'IBM Plex Sans',
                provider: 'local',
                src: '/fonts/IBMPlexSans-SemiBold.ttf',
                display: 'swap',
                weight: 600,
                style: 'normal',
                fallbacks: ['Arial']
            }
        ],
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
            link: [
                {
                    rel: 'preload',
                    as: 'font',
                    type: 'font/ttf',
                    href: '/fonts/CaesarDressing-Regular.ttf',
                    crossorigin: 'anonymous',
                },
                {
                    rel: 'preload',
                    as: 'font',
                    type: 'font/ttf',
                    href: '/fonts/IBMPlexSans-Regular.ttf',
                    crossorigin: 'anonymous',
                },
                {
                    rel: 'preload',
                    as: 'font',
                    type: 'font/ttf',
                    href: '/fonts/IBMPlexSans-SemiBold.ttf',
                    crossorigin: 'anonymous',
                },
            ],
        },
    },

    modules: ['@nuxt/fonts', 'nuxt-telegram-auth'],
    runtimeConfig: {
        // Этот токен доступен только на сервере
        TELEGRAM_TOKEN: process.env.TELEGRAM_TOKEN
    }
})