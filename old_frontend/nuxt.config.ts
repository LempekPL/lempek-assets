// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    compatibilityDate: '2024-11-01',
    ssr: false,
    css: [
        '@/public/css/global.css'
    ],
    devtools: {enabled: true},
    modules: ['@nuxt/icon', '@nuxt/image', '@pinia/nuxt'],
    runtimeConfig: {
        public: {
            apiBase: 'http://localhost:7001'
        }
    },
    devServer: {
        port: 7002
    }
})