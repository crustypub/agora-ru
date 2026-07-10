/**
 * chat.client.ts — глобальная инициализация чат-сессии.
 *
 * Запускается один раз при старте клиентского приложения.
 * Следит за состоянием авторизации и управляет жизненным циклом WS:
 *   - authUser → есть  → connectWs() + fetchRooms()
 *   - authUser → null  → disconnectWs() + resetState()
 *
 * Также выступает основой для push-уведомлений о новых сообщениях
 * когда пользователь находится вне страницы /chats.
 */
import { watch } from 'vue';
import { useAuthUser } from '~/composables/useAuthUser';
import { useChat } from '~/composables/useChat';
import { useRoute } from '#app';

export default defineNuxtPlugin(() => {
  const authUser = useAuthUser();
  const route = useRoute();
  const { connectWs, disconnectWs, resetState, fetchRooms, rooms, isWsConnected } = useChat();

  // Реагируем на изменение состояния авторизации
  watch(
    authUser,
    (user, prevUser) => {
      if (user) {
        // Пользователь вошёл или приложение загружено с активной сессией
        connectWs();
        // Загружаем список комнат если ещё не загружен (silent=true — без лоадера)
        if (rooms.value.length === 0) {
          fetchRooms(1, true);
        }
      } else if (prevUser) {
        // Пользователь вышел — закрываем соединение и чистим состояние
        disconnectWs();
        resetState();
      }
    },
    { immediate: true }
  );

  // TODO: Базовый хук для push-уведомлений о новых сообщениях.
  // Место для расширения: смотреть на изменение rooms.value[x].unread_count
  // и вызывать useNotify().success() если route.path не начинается с '/chats'.
  //
  // Пример расширения:
  // watch(
  //   () => rooms.value.map(r => r.unread_count).reduce((a, b) => a + b, 0),
  //   (newTotal, oldTotal) => {
  //     if (newTotal > oldTotal && !route.path.startsWith('/chats')) {
  //       const notify = useNotify();
  //       notify.success('Новое сообщение', 'У вас непрочитанные сообщения в чатах');
  //     }
  //   }
  // );
});
