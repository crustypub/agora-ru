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

// ─── Link Preview helpers ──────────────────────────────────────────────────────

const LP_SEPARATOR = '\n\n---LINK_PREVIEW---';

export interface ILinkPreviewData {
  url: string;
  title: string;
  desc?: string;
  img?: string;
}

/**
 * Парсит сырое сообщение: отделяет пользовательский текст от JSON-метаданных превью.
 * Если разделитель отсутствует или JSON невалиден — возвращает весь текст как есть.
 */
export function parseMessageContent(raw: string): { text: string; preview: ILinkPreviewData | null } {
  if (!raw) return { text: '', preview: null };
  const idx = raw.indexOf(LP_SEPARATOR);
  if (idx === -1) return { text: raw, preview: null };
  const text = raw.slice(0, idx);
  try {
    const preview = JSON.parse(raw.slice(idx + LP_SEPARATOR.length)) as ILinkPreviewData;
    return { text, preview };
  } catch {
    return { text: raw, preview: null };
  }
}

/**
 * Кодирует текст + превью в строку для отправки на сервер.
 */
export function encodeMessageContent(text: string, preview: ILinkPreviewData | null): string {
  if (!preview) return text;
  return `${text}${LP_SEPARATOR}${JSON.stringify(preview)}`;
}

/**
 * Возвращает только текстовую часть сообщения (без JSON-хвоста).
 * Используется для превью в списке комнат и копирования.
 */
export function stripLinkPreview(raw: string): string {
  if (!raw) return '';
  const idx = raw.indexOf(LP_SEPARATOR);
  return idx === -1 ? raw : raw.slice(0, idx);
}

/** Простой regex-поиск первого URL в строке. */
export function extractFirstUrl(text: string): string | null {
  const match = text.match(/https?:\/\/[^\s]+/);
  return match ? match[0] : null;
}
