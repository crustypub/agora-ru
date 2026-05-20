import type { IUserResponse } from "~/models/entities/user.entities";

export const useAuthUser = () => {
    return useState<IUserResponse | null>('auth_user', () => null);
};
