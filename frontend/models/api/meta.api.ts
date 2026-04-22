export interface IMetaPagination {
    current_page: number,
    has_next: boolean,
    has_previous: boolean,
    per_page: number,
    total_count: number,
    total_pages: number
}