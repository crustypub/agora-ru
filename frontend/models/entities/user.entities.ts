export interface IUserResponse {
  id: string;
  username: string;
  first_name: string | null;
  last_name: string | null;
  avatar_url: string | null;
  telegram_id: number | null;
  created_at: number;
}
