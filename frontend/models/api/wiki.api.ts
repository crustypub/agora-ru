import type {
  IWikiArticle,
  IWikiArticleSimple,
  IWikiType,
} from "../entities/wiki.entities";
import type { IMetaPagination } from "./meta.api";
import type { TStatus } from "./status.api";

export interface IWikiTypesResponse {
  data: IWikiType[];
  status: TStatus;
}

export interface IWikiArticleResponse {
  data: IWikiArticle;
  status: TStatus;
}

export interface IWikiResponse {
  data: IWikiArticleSimple[];
  status: TStatus;
  meta: IMetaPagination;
}
