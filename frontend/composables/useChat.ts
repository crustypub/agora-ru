import { computed } from 'vue';
import type {
  IChatListItem,
  IChatMessage,
  IRoomMemberInfo,
  WsIncomingEvent,
  WsOutgoingEvent,
  IPaginatedResponse,
  IRoomMessagesResponse
} from '~/models/entities/chat.entities';
import { useAuthUser } from './useAuthUser';
import { useApiCall } from './useApi';

// Синглтон-переменные уровня модуля: делятся между всеми вызовами useChat()
let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;

export const useChat = () => {
  const authUser = useAuthUser();

  // Глобальное реактивное состояние через Nuxt useState (SSR-safe singleton)
  const rooms = useState<IChatListItem[]>('chat_rooms', () => []);
  const activeRoomId = useState<string | null>('active_room_id', () => null);
  // Переименовано в messagesStore, чтобы отличать от экспортируемого computed<IChatMessage[]>
  const messagesStore = useState<Record<string, IChatMessage[]>>('chat_messages', () => ({}));

  // Кэшируем участников для каждой комнаты отдельно, чтобы избежать багов с отображением участников предыдущей комнаты
  const roomMembersStore = useState<Record<string, IRoomMemberInfo[]>>('chat_room_members', () => ({}));
  const activeRoomMembers = computed<IRoomMemberInfo[]>(() => {
    return activeRoomId.value ? roomMembersStore.value[activeRoomId.value] || [] : [];
  });

  const isWsConnected = useState<boolean>('chat_ws_connected', () => false);
  const roomsLoading = ref(false);
  const messagesLoading = ref(false);

  const roomsMeta = useState<any>('chat_rooms_meta', () => null);
  const messagesMeta = useState<Record<string, any>>('chat_messages_meta', () => ({}));

  // --- WebSocket ---

  const connectWs = (isReconnect = false) => {
    if (import.meta.server || ws) return;
    // Сбрасываем счётчик ретраев при явном вызове (например, после повторной авторизации)
    if (!isReconnect) {
      reconnectAttempts = 0;
    }

    const config = useRuntimeConfig();
    const apiBase = config.public.apiBase;
    
    let wsUrl = '';
    if (apiBase && (apiBase.startsWith('http://') || apiBase.startsWith('https://'))) {
      let url = apiBase.replace(/^http/, 'ws') + '/api/ws/chat';
      // Если мы в браузере, подменяем localhost/127.0.0.1 на текущий hostname из window.location,
      // чтобы совпадали домены для кук (куки с localhost не шлются на 127.0.0.1 и наоборот)
      if (typeof window !== 'undefined') {
        const currentHost = window.location.hostname;
        url = url.replace('127.0.0.1', currentHost).replace('localhost', currentHost);
      }
      wsUrl = url;
    } else {
      const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      wsUrl = `${wsProto}//${window.location.host}/api/ws/chat`;
    }

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      isWsConnected.value = true;
      reconnectAttempts = 0; // сбрасываем счётчик при успешном подключении
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      // При реконнекте — уведомляем бэкенд о прочтении текущей комнаты
      if (activeRoomId.value) {
        readRoom(activeRoomId.value);
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg: WsIncomingEvent = JSON.parse(event.data);
        handleIncomingWsMessage(msg);
      } catch (e) {
        console.error('Ошибка обработки входящего WS-сообщения:', e);
      }
    };

    ws.onclose = () => {
      isWsConnected.value = false;
      ws = null;
      // Exponential backoff: 1s, 2s, 4s, 8s ... до 30s, не более MAX_RECONNECT_ATTEMPTS попыток
      if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        const delay = Math.min(1000 * 2 ** reconnectAttempts, 30_000);
        reconnectAttempts++;
        reconnectTimer = setTimeout(() => connectWs(true), delay);
      } else {
        console.warn('WebSocket: превышен лимит попыток переподключения. Соединение не восстановлено.');
      }
    };

    ws.onerror = (err) => {
      console.error('Ошибка соединения WebSocket:', err);
    };
  };

  const disconnectWs = () => {
    // Сброс счётчика при явном отключении — чтобы следующий connectWs начал с нуля
    reconnectAttempts = MAX_RECONNECT_ATTEMPTS;
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (ws) {
      ws.close();
      ws = null;
    }
    isWsConnected.value = false;
  };

  // --- Обработка входящих WS-событий ---

  const handleIncomingWsMessage = (msg: WsIncomingEvent) => {
    switch (msg.event) {
      case 'new_message': {
        const payload = msg.payload;

        const newMsg: IChatMessage = {
          id: payload.id,
          room_id: payload.room_id,
          sender_id: payload.sender_id,
          content: payload.content,
          created_at: Math.floor(new Date(payload.created_at).getTime() / 1000),
          author: null
        };

        // Находим информацию об авторе из участников активной комнаты
        const authorInfo = (activeRoomMembers.value || []).find(m => m.id === payload.sender_id);
        if (authorInfo) {
          newMsg.author = {
            id: authorInfo.id,
            username: authorInfo.username,
            first_name: authorInfo.first_name,
            last_name: authorInfo.last_name,
            avatar_url: authorInfo.avatar_url,
            telegram_id: null,
            created_at: 0
          };
        }

        // 1. Добавляем в историю, предотвращая дублирование
        if (!messagesStore.value[payload.room_id]) {
          messagesStore.value[payload.room_id] = [];
        }
        if (!messagesStore.value[payload.room_id].some(m => m.id === newMsg.id)) {
          messagesStore.value[payload.room_id].push(newMsg);
        }

        // 2. Сбрасываем счётчик непрочитанных если комната активна, иначе — инкрементируем
        if (activeRoomId.value === payload.room_id) {
          readRoom(payload.room_id);
        } else {
          const room = rooms.value.find(r => r.id === payload.room_id);
          if (room) {
            room.unread_count++;
          }
        }

        // 3. Обновляем превью и поднимаем комнату вверх списка
        const roomIndex = rooms.value.findIndex(r => r.id === payload.room_id);
        if (roomIndex !== -1) {
          const room = rooms.value[roomIndex];
          room.last_message = newMsg;
          rooms.value.splice(roomIndex, 1);
          rooms.value.unshift(room);
        } else {
          // Нас добавили в новый чат — перезагружаем список
          fetchRooms(1, true);
        }
        break;
      }

      case 'message_deleted': {
        const { message_id, room_id } = msg.payload;

        if (messagesStore.value[room_id]) {
          messagesStore.value[room_id] = messagesStore.value[room_id].filter(m => m.id !== message_id);
        }

        // Обновляем превью last_message в списке комнат
        const room = rooms.value.find(r => r.id === room_id);
        if (room && room.last_message?.id === message_id) {
          const roomMsgs = messagesStore.value[room_id] || [];
          room.last_message = roomMsgs[roomMsgs.length - 1] || null;
        }
        break;
      }

      case 'room_created': {
        fetchRooms(1, true);
        break;
      }

      case 'room_read': {
        const { room_id, user_id, last_read_at } = msg.payload;

        const msgs = messagesStore.value[room_id];
        if (msgs) {
          msgs.forEach(m => {
            if (m.sender_id !== user_id && m.created_at <= last_read_at) {
              m.is_read = true;
            }
          });
        }

        const room = rooms.value.find(r => r.id === room_id);
        if (room && room.last_message) {
          if (room.last_message.sender_id !== user_id && room.last_message.created_at <= last_read_at) {
            room.last_message.is_read = true;
          }
        }
        break;
      }
    }
  };

  // --- Отправка WS-событий ---

  const sendWsEvent = (event: WsOutgoingEvent) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(event));
    } else {
      console.warn('WebSocket не подключен. Событие не отправлено:', event);
    }
  };

  // --- REST API ---

  const fetchRooms = async (page = 1, silent = false, search = '') => {
    if (!silent) roomsLoading.value = true;
    try {
      const params: any = { page, limit: 20 };
      if (search && search.trim()) {
        params.search_value = search.trim();
      }
      const response = await useApiCall<IPaginatedResponse<IChatListItem>>('/api/chats', {
        params
      });

      if (page === 1) {
        rooms.value = response.data;
      } else {
        const existingIds = new Set(rooms.value.map(r => r.id));
        const newRooms = response.data.filter(r => !existingIds.has(r.id));
        rooms.value = [...rooms.value, ...newRooms];
      }
      roomsMeta.value = response.meta;
    } catch (e) {
      console.error('Ошибка загрузки списка чатов:', e);
    } finally {
      roomsLoading.value = false;
    }
  };

  const fetchMessages = async (roomId: string, page = 1) => {
    messagesLoading.value = true;
    try {
      const response = await useApiCall<IRoomMessagesResponse>(`/api/chats/${roomId}`, {
        params: { page, limit: 30 }
      });

      // Защита от race condition: если пользователь сменил комнату пока шёл запрос — игнорируем ответ
      if (activeRoomId.value !== roomId) return;

      if (page === 1) {
        roomMembersStore.value[roomId] = response.members;
      } else {
        const currentMembers = roomMembersStore.value[roomId] || [];
        const existingMemberIds = new Set(currentMembers.map(m => m.id));
        const newMembers = response.members.filter(m => !existingMemberIds.has(m.id));
        roomMembersStore.value[roomId] = [...currentMembers, ...newMembers];
      }

      // Сообщения приходят DESC с бэкенда — переворачиваем для хронологии
      const fetchedMsgs = [...response.data].reverse();

      if (page === 1) {
        messagesStore.value[roomId] = fetchedMsgs;
      } else {
        const existingMsgIds = new Set((messagesStore.value[roomId] || []).map(m => m.id));
        const uniqueFetched = fetchedMsgs.filter(m => !existingMsgIds.has(m.id));
        messagesStore.value[roomId] = [...uniqueFetched, ...(messagesStore.value[roomId] || [])];
      }

      messagesMeta.value[roomId] = response.meta;
    } catch (e) {
      console.error('Ошибка загрузки сообщений:', e);
    } finally {
      messagesLoading.value = false;
    }
  };

  const sendMessage = (content: string) => {
    if (!activeRoomId.value || !content.trim()) return;
    sendWsEvent({
      event: 'send_message',
      payload: { room_id: activeRoomId.value, content: content.trim() }
    });
  };

  const readRoom = (roomId: string) => {
    sendWsEvent({
      event: 'read_room',
      payload: { room_id: roomId }
    });
    const room = rooms.value.find(r => r.id === roomId);
    if (room) {
      room.unread_count = 0;
    }
  };

  const selectRoom = (roomId: string | null) => {
    activeRoomId.value = roomId;
    if (roomId) {
      if (!messagesMeta.value[roomId]) {
        fetchMessages(roomId, 1);
      } else {
        readRoom(roomId);
      }
    }
  };

  const createDirectChat = async (userId: string) => {
    try {
      const response = await useApiCall<any>('/api/chats', {
        method: 'POST',
        body: {
          room_type: 'Direct',
          direct_user_id: userId
        }
      });
      const roomId = response.data.room_id;
      await fetchRooms(1, true);
      selectRoom(roomId);
      return { success: true, roomId };
    } catch (e: any) {
      console.error('Ошибка создания личного чата:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  const createGroupChat = async (name: string, description?: string) => {
    try {
      const response = await useApiCall<any>('/api/chats', {
        method: 'POST',
        body: {
          room_type: 'Group',
          name,
          description: description || null
        }
      });
      const roomId = response.data.room_id;
      await fetchRooms(1, true);
      selectRoom(roomId);
      return { success: true, roomId };
    } catch (e: any) {
      console.error('Ошибка создания группового чата:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  const deleteMessage = async (messageId: string, deleteType: 'me' | 'everyone') => {
    try {
      await useApiCall(`/api/messages/${messageId}`, {
        method: 'DELETE',
        query: { type: deleteType }
      });

      if (activeRoomId.value && messagesStore.value[activeRoomId.value]) {
        messagesStore.value[activeRoomId.value] = messagesStore.value[activeRoomId.value].filter(m => m.id !== messageId);

        const room = rooms.value.find(r => r.id === activeRoomId.value);
        if (room && room.last_message?.id === messageId) {
          const roomMsgs = messagesStore.value[activeRoomId.value] || [];
          room.last_message = roomMsgs[roomMsgs.length - 1] || null;
        }
      }
      return { success: true };
    } catch (e: any) {
      console.error('Ошибка удаления сообщения:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  const inviteMember = async (roomId: string, userId: string) => {
    try {
      await useApiCall(`/api/chats/${roomId}/members`, {
        method: 'POST',
        body: { user_id: userId }
      });
      // TODO: Идеально — отдельный GET /api/chats/{id}/members.
      // Пока перезагружаем участников через fetchMessages(page=1).
      // Это сбрасывает кеш сообщений для данной комнаты.
      await fetchMessages(roomId, 1);
      return { success: true };
    } catch (e: any) {
      console.error('Ошибка добавления участника:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  const kickMember = async (roomId: string, userId: string) => {
    try {
      await useApiCall(`/api/chats/${roomId}/members/${userId}`, {
        method: 'DELETE'
      });
      if (roomMembersStore.value[roomId]) {
        roomMembersStore.value[roomId] = roomMembersStore.value[roomId].filter(m => m.id !== userId);
      }
      return { success: true };
    } catch (e: any) {
      console.error('Ошибка удаления участника:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  const leaveRoom = async (roomId: string) => {
    if (!authUser.value) return { success: false };
    const myId = authUser.value.id;
    try {
      await useApiCall(`/api/chats/${roomId}/members/${myId}`, {
        method: 'DELETE'
      });
      selectRoom(null);
      await fetchRooms(1, true);
      return { success: true };
    } catch (e: any) {
      console.error('Ошибка выхода из чата:', e);
      return { success: false, error: e.data?.error || e.message };
    }
  };

  // Полный сброс состояния чатов (вызывается при logout)
  const resetState = () => {
    rooms.value = [];
    activeRoomId.value = null;
    messagesStore.value = {};
    roomMembersStore.value = {};
    roomsMeta.value = null;
    messagesMeta.value = {};
    isWsConnected.value = false;
  };

  const totalUnreadCount = computed(() => {
    return rooms.value.reduce((acc, room) => acc + (room.unread_count || 0), 0);
  });

  return {
    rooms,
    activeRoomId,
    messages: computed(() => (activeRoomId.value ? messagesStore.value[activeRoomId.value] || [] : [])),
    members: activeRoomMembers,
    roomsLoading,
    messagesLoading,
    isWsConnected,
    roomsMeta,
    messagesMeta: computed(() => (activeRoomId.value ? messagesMeta.value[activeRoomId.value] || null : null)),
    totalUnreadCount,

    connectWs,
    disconnectWs,
    resetState,
    fetchRooms,
    fetchMessages,
    sendMessage,
    readRoom,
    selectRoom,
    createDirectChat,
    createGroupChat,
    deleteMessage,
    inviteMember,
    kickMember,
    leaveRoom
  };
};
