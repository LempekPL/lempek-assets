// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  ssr: false,
  css: [
    './public/css/global.css'
  ],
  devtools: { enabled: true },
  modules: ['@nuxt/icon', '@nuxt/image', '@pinia/nuxt'],
  runtimeConfig: {
    public: {
      apiBase: process.env.BACKEND_URL || 'http://localhost:7001/api',
    }
  },
  devServer: {
    port: 7002,
  }
})