import { useAuthUser } from "./useAuthUser";
import { useApiCall } from "./useApi";
import type { IUser } from "~/models/entities/user.entities";

export const useCheckAuth = () => {
  const authUser = useAuthUser();

  const checkAuth = async (force: boolean = false): Promise<IUser | null> => {
    // 1. If already resolved in state and not forcing a refresh, return cached user
    if (authUser.value && !force) {
      return authUser.value;
    }

    // 2. Fetch the current authenticated user profile
    try {
      const response = await useApiCall<{ data: IUser }>('/api/auth/me');
      if (response && response.data) {
        authUser.value = response.data;
        return response.data;
      }
    } catch (e) {
      // Clear state since the call failed (invalid or absent token)
      authUser.value = null;
    }

    return null;
  };

  return {
    checkAuth,
    authUser,
  };
};
