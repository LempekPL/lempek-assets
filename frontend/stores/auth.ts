import {defineStore} from "pinia"

export type UserData = {
    user_id: string,
    login: string,
    allow_upload: boolean
}

export const useAuthStore = defineStore('auth', {
    state: () => ({
        user: null as UserData | null,
        loading: false
    }),

    getters: {
        isAuthenticated: (state) => !!state.user,
    },

    actions: {
        async fetchUser() {
            this.loading = true
            try {
                const config = useRuntimeConfig();
                this.user = await $fetch(config.public.apiBase + '/user', {
                    credentials: 'include'
                });
            } catch (error) {
                this.user = null
            } finally {
                this.loading = false
            }
        },

        async login(credentials: { login: string; password: string }): Promise<{success: boolean, message: string}> {
            const config = useRuntimeConfig();
            return await $fetch(config.public.apiBase+'/login', {
                method: 'POST',
                credentials: 'include',
                body: credentials
            })
        },

        async register(credentials: { login: string; password: string }): Promise<{success: boolean, message: string}> {
            const config = useRuntimeConfig();
            return await $fetch(config.public.apiBase+'/register', {
                method: 'POST',
                credentials: 'include',
                body: credentials
            })
        },

        async logout() {
            const config = useRuntimeConfig();
            await $fetch(config.public.apiBase+'/logout', {
                method: 'POST',
                credentials: 'include'
            })
            this.user = null
        }
    }
})