/**
 * Утилиты для форматирования данных чатов.
 * Централизует логику, которая ранее дублировалась в ChatRoomItem, ChatArea,
 * ChatDetailsModal и ChatMessageItem.
 */

interface UserLike {
  first_name?: string | null;
  last_name?: string | null;
  username: string;
}

/**
 * Возвращает отображаемое имя пользователя.
 * Приоритет: имя + фамилия > username.
 */
export const formatUserName = (user: UserLike): string => {
  if (user.first_name) {
    return `${user.first_name} ${user.last_name || ''}`.trim();
  }
  return user.username;
};

/**
 * Форматирует Unix timestamp (секунды) для отображения в списке комнат:
 * - Сегодня → «ЧЧ:ММ»
 * - Вчера → «Вчера»
 * - Старше → «ДД.ММ»
 */
export const formatRoomTime = (timestamp: number): string => {
  if (!timestamp) return '';

  const date = new Date(timestamp * 1000);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const compareDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

  if (compareDate.getTime() === today.getTime()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (compareDate.getTime() === yesterday.getTime()) {
    return 'Вчера';
  } else {
    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    return `${day}.${month}`;
  }
};

/**
 * Форматирует Unix timestamp (секунды) для разделителя дат в ленте сообщений:
 * - Сегодня → «Сегодня»
 * - Вчера → «Вчера»
 * - Старше → «DD MMMM YYYY»
 */
export const formatDateDivider = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const compareDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

  if (compareDate.getTime() === today.getTime()) {
    return 'Сегодня';
  } else if (compareDate.getTime() === yesterday.getTime()) {
    return 'Вчера';
  } else {
    return date.toLocaleDateString([], { day: 'numeric', month: 'long', year: 'numeric' });
  }
};


export const formatMessageTime = (timestamp: number): string => {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
};
