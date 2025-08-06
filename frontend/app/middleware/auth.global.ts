import {useAuthStore} from "~/stores/auth";

export default defineNuxtRouteMiddleware(async (to) => {
    const auth = useAuthStore();
    const publicRoutes = ['/login', '/register'];
    await auth.fetchUser();

    if (!auth.isAuthenticated && !publicRoutes.includes(to.path)) {
        return navigateTo('/login');
    }
    if (auth.isAuthenticated && publicRoutes.includes(to.path)) {
        return navigateTo('/');
    }
})