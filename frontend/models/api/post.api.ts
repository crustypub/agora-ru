import type { IPostResponseItem } from "../entities/post.entities";
import type { IMetaPagination } from "./meta.api";
import type { TStatus} from "./status.api";

export interface IPostResponse {
    data: IPostResponseItem[],
    meta: IMetaPagination,
    status: TStatus,
}