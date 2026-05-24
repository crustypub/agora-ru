import type { IUserResponse } from "./user.entities";

export interface IWikiType {
  id: number;
  title: string;
  created_at: number;
  updated_at: number;
}

export interface IWikiArticle {
  title: string;
  content: string;
  created_at: number;
  created_by: IUserResponse;
  id: string;
  is_confirmed: boolean;
  last_edited_by: IUserResponse;
  updated_at: number;
  wiki_type: IWikiType;
  stars_count: number;
  comment_count: number;
  is_starred: boolean;
}

export type IWikiArticleSimple = Omit<IWikiArticle, "content">;
