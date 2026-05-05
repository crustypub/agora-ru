
import type { IWikiTypeResponseItem } from "../entities/wiki.entities";
import type { TStatuse } from "./status.api";

export interface IWikiTypesResponse {
    data: IWikiTypeResponseItem[],
    status: TStatuse,
}