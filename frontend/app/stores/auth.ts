import {defineStore} from "pinia"
import {type AuthUser} from "~~/types/user";
import type {ApiResponse} from "~~/types/api";

export const useAuthStore = defineStore('auth', {
    state: () => ({
        user: null as AuthUser | null,
        loading: false,
    }),

    getters: {
        isAuthenticated: (state) => !!state.user,
    },

    actions: {
        async fetchUser() {
            this.loading = true;
            try {
                const config = useRuntimeConfig();
                this.user = await $fetch<AuthUser>(config.public.apiBase + '/user', {
                    credentials: 'include',
                });
            } catch {
                this.user = null;
            } finally {
                this.loading = false;
            }
        },

        async login(credentials: { login: string; password: string }): Promise<ApiResponse> {
            const config = useRuntimeConfig();
            try {
                return await $fetch<ApiResponse>(config.public.apiBase + '/login', {
                    method: 'POST',
                    credentials: 'include',
                    body: credentials
                });
            } catch (error: any) {
                if (error?.data) {
                    return error.data as ApiResponse;
                }
                return {
                    success: false,
                    detail: 'Nie udało się zalogować (błąd sieci).',
                    err_id: null
                };
            }
        },

        async changePassword(credentials: { current_password: string, new_password: string }): Promise<ApiResponse> {
            const config = useRuntimeConfig();
            try {
                return await $fetch<ApiResponse>(config.public.apiBase + '/user/password', {
                    method: 'PATCH',
                    credentials: 'include',
                    body: credentials
                });
            } catch (error: any) {
                if (error?.data) {
                    return error.data as ApiResponse;
                }
                return {
                    success: false,
                    detail: 'Nie udało się zmienić hasła (błąd sieci).',
                    err_id: null
                };
            }
        },

        async changeUsername(credentials: { password: string, new_username: string }): Promise<ApiResponse> {
            const config = useRuntimeConfig();
            try {
                return await $fetch<ApiResponse>(config.public.apiBase + '/user/username', {
                    method: 'PATCH',
                    credentials: 'include',
                    body: credentials
                });
            } catch (error: any) {
                if (error?.data) {
                    return error.data as ApiResponse;
                }
                return {
                    success: false,
                    detail: 'Nie udało się zmienić nazwy (błąd sieci).',
                    err_id: null
                };
            }
        },

        async logout() {
            const config = useRuntimeConfig();
            await $fetch(config.public.apiBase + '/logout', {
                method: 'POST',
                credentials: 'include'
            })
            this.user = null
        }
    }
})