import {useAuthStore} from "~/stores/auth";
const publicRoutes = ['/changelog', '/login'];
const notLoggedRoutes = ['/login'];

export default defineNuxtRouteMiddleware(async (to) => {
    const auth = useAuthStore();
    await auth.fetchUser();

    if (!auth.isAuthenticated && !publicRoutes.includes(to.path)) {
        return navigateTo('/login');
    }
    if (auth.isAuthenticated && notLoggedRoutes.includes(to.path)) {
        return navigateTo('/');
    }
})