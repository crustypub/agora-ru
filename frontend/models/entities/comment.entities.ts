import type { IUserResponse } from "./user.entities";

export interface IComment {
    id: string;
    entity_type: string;
    entity_id: string;
    author: IUserResponse;
    content: string;
    created_at: number;
    updated_at: number;
}
