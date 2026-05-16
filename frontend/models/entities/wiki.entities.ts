export interface IWikiTypeResponseItem {
    id: number,
    title: string,
    created_at: number,
    updated_at: number,
}

export interface IWikiArticleResponeItem {
    content: string,
    created_at: number,
    created_by: string,
    id: string,
    is_confirmed: boolean,
    last_edited_by: string,
    title: string,
    updated_at: number,
    wiki_type: IWikiTypeResponseItem,
}
