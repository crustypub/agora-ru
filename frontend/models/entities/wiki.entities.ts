import type { IUser } from "./user.entities";

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
  created_by: IUser;
  id: string;
  is_confirmed: boolean;
  last_edited_by: IUser;
  updated_at: number;
  wiki_type: IWikiType;
  stars_count: number;
  comment_count: number;
  is_starred: boolean;
}

export type IWikiArticleSimple = Omit<IWikiArticle, "content">;
