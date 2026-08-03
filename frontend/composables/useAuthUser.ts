import type { IUser } from "~/models/entities/user.entities";

export const useAuthUser = () => {
    return useState<IUser | null>('auth_user', () => null);
};
