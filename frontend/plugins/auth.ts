import { useAuthUser } from "~/composables/useAuthUser";
import { useApi } from "~/composables/useApi";
import type { IUserResponse } from "~/models/entities/user.entities";

export default defineNuxtPlugin(async (nuxtApp) => {
    const authUser = useAuthUser();

    try {
        const { data } = await useApi<{ data: IUserResponse }>('/api/auth/me');
        if (data.value && data.value.data) {
            authUser.value = data.value.data;
        }
    } catch (e) {
        // Not logged in or token expired
        authUser.value = null;
    }
});
