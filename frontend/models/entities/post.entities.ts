export interface IPostAuthor {
    "id": string,
    "username": null | string,
    "first_name": null | string,
    "last_name": null | string,
    "avatar_url": null | string,
}

export interface IPostResponseItem {
    id: string,
    author: IPostAuthor,
    comments_count: number,
    title: string,
    content: string,
    rating_minus: number,
    rating_plus: number
    created_at: number,
    updated_at: number,
}