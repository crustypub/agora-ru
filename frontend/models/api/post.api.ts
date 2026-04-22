import type { IPostResponseItem } from "../entities/post.entities";
import type { IMetaPagination } from "./meta.api";
import type { TStatuse } from "./status.api";

export interface IPostResponse {
    data: IPostResponseItem[],
    meta: IMetaPagination,
    status: TStatuse,
}