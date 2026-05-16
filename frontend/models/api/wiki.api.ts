
import type { IWikiArticleResponeItem, IWikiTypeResponseItem } from "../entities/wiki.entities";
import type { TStatus } from "./status.api";

export interface IWikiTypesResponse {
    data: IWikiTypeResponseItem[],
    status: TStatus,
}

export interface IWikiArticleResponse {
    data: IWikiArticleResponeItem,
    status: TStatus
}