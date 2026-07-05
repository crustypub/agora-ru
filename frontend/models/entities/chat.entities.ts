import type { IUserResponse } from "./user.entities";

export type ChatRoomType = 'direct' | 'group';
export type RoomRole = 'owner' | 'moderator' | 'member';

export interface IChatAttachment {
  id: string;
  file_name: string;
  file_size: number;
  file_mime: string;
  file_url: string;
}

export interface IChatMessage {
  id: string;
  room_id: string;
  sender_id: string | null;
  content: string;
  created_at: number; // Unix Epoch в секундах
  author: IUserResponse | null;
  is_read?: boolean;
  attachments?: IChatAttachment[] | null;
}

export interface IChatListItem {
  id: string;
  room_type: ChatRoomType;
  name: string | null;
  description: string | null;
  direct_key: string | null;
  created_at: number;
  updated_at: number;
  unread_count: number;
  last_message: IChatMessage | null;
  direct_user: IUserResponse | null;
}

export interface IRoomMemberInfo {
  id: string;
  username: string;
  first_name: string | null;
  last_name: string | null;
  avatar_url: string | null;
  role: RoomRole;
}

// Ответы REST API
export interface IPaginatedResponse<T> {
  status: string;
  data: T[];
  meta: {
    current_page: number;
    per_page: number;
    total_count: number;
    total_pages: number;
    has_next: boolean;
    has_previous: boolean;
  };
}

export interface IRoomMessagesResponse extends IPaginatedResponse<IChatMessage> {
  members: IRoomMemberInfo[];
}

// WebSocket события
export type WsOutgoingEvent =
  | { event: 'send_message'; payload: { room_id: string; content: string; attachments?: { file_key: string; file_name: string; file_size: number; file_mime: string }[] | null } }
  | { event: 'read_room'; payload: { room_id: string } };

export type WsIncomingEvent =
  | { event: 'new_message'; payload: { id: string; room_id: string; sender_id: string; content: string; created_at: string; attachments?: IChatAttachment[] | null } }
  | { event: 'message_deleted'; payload: { message_id: string; room_id: string } }
  | { event: 'room_created'; payload: { room_id: string } }
  | { event: 'room_read'; payload: { room_id: string; user_id: string; last_read_at: number } };
