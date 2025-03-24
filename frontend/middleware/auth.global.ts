import {useAuthStore} from "~/stores/auth";

export default defineNuxtRouteMiddleware(async (to) => {
    const auth = useAuthStore();
    try {
        if (!auth.user) {
            await auth.fetchUser();
        }

        if (!auth.isAuthenticated && to.path !== '/login' && to.path !== '/register') {
            return navigateTo('/login');
        }

        if (auth.isAuthenticated && (to.path === '/login' || to.path === '/register')) {
            return navigateTo('/');
        }
    } catch (error) {
        console.error('Auth middleware error:', error);
        auth.$reset();
    }
})