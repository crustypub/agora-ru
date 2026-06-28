<template>
  <div class="chat-area">
    <div v-if="!activeRoomId" class="chat-area__empty">
      <UIcon name="material-symbols:chat-bubble-outline" class="chat-area__empty-icon" />
      <h2 class="chat-area__empty-title">Ваша переписка</h2>
      <p class="chat-area__empty-text">
        Выберите существующий чат из списка слева или создайте новый, чтобы начать общение.
      </p>
    </div>

    <template v-else>
      <div class="chat-area__header">
        <div class="chat-area__header-left">
          <UButton
            v-if="isMobile"
            icon="material-symbols:arrow-back-rounded"
            variant="ghost"
            color="neutral"
            size="sm"
            class="chat-area__back-btn"
            @click="$emit('back')"
          />

          <UAvatar
            :src="avatarUrl || ''"
            :alt="roomName"
            :class="isDirect ? 'chat-area__avatar--direct' : 'chat-area__avatar--group'"
            size="sm"
            class="chat-area__avatar"
          >
            <template v-if="!avatarUrl">
              {{ roomName.charAt(0).toUpperCase() }}
            </template>
          </UAvatar>

          <div class="chat-area__header-info">
            <span class="chat-area__header-name">
              {{ roomName }}
            </span>
            <span class="chat-area__header-meta">
              {{ isDirect ? 'Личный диалог' : `${members.length} участников` }}
            </span>
          </div>
        </div>

        <!-- Header Actions -->
        <div class="chat-area__header-actions">
          <UButton
            icon="material-symbols:info-outline"
            variant="ghost"
            color="neutral"
            size="sm"
            title="Информация о чате"
            @click="isDetailsModalOpen = true"
          />
        </div>
      </div>

      <div
        ref="feedContainer"
        class="chat-area__feed"
        @scroll="handleScroll"
      >
        <div v-if="isLoadingMore" class="chat-area__loader chat-area__loader--pagination">
          <UIcon name="material-symbols:sync-saved-locally-outline-rounded" class="chat-area__spinner" />
        </div>

        <div v-if="messagesLoading && messages.length === 0" class="chat-area__loader">
          <UIcon name="material-symbols:sync-saved-locally-outline-rounded" class="chat-area__spinner" />
        </div>

        <div v-else-if="messages.length === 0 && !messagesLoading" class="chat-area__no-messages">
          <span>История переписки пуста.</span>
          <span>Напишите первое сообщение ниже!</span>
        </div>

        <template v-else>
          <template v-for="(msg, idx) in messages" :key="msg.id">
            <div
              v-if="shouldShowDateDivider(msg, idx)"
              class="chat-area__date-divider"
            >
              <span class="chat-area__date-text">
                {{ formatDateDivider(msg.created_at) }}
              </span>
            </div>

            <!-- display:contents делает обёртку прозрачной для flex-layout родителя,
                 но сохраняет data-msg-id в DOM для scroll-якоря при пагинации -->
            <div :data-msg-id="msg.id" style="display: contents">
              <ChatMessageItem
                :message="msg"
                :is-me="msg.sender_id === currentUserId"
                :is-group="!isDirect"
                :can-delete-for-everyone="canDeleteForEveryone(msg)"
                @delete="deleteMessage"
              />
            </div>
          </template>
        </template>
      </div>

      <div class="chat-area__input-bar">
        <UTextarea
          v-model="inputText"
          placeholder="Напишите сообщение..."
          autoresize
          :rows="1"
          max-rows="4"
          class="chat-area__textarea"
          @keydown.enter="handleEnterKey"
        />
        <UButton
          icon="material-symbols:send-rounded"
          color="primary"
          size="md"
          class="chat-area__send-btn"
          :disabled="!inputText.trim()"
          @click="handleSendMessage"
        />
      </div>
    </template>

    <!-- Details modal -->
    <ChatDetailsModal
      v-model:open="isDetailsModalOpen"
      :room="currentRoom"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { useChat } from '~/composables/useChat';
import { useAuthUser } from '~/composables/useAuthUser';
import type { IChatMessage } from '~/models/entities/chat.entities';
import { formatDateDivider, formatUserName } from '~/helpers/chat';
import ChatMessageItem from './ChatMessageItem.vue';
import ChatDetailsModal from './ChatDetailsModal.vue';

const props = defineProps<{
  isMobile: boolean;
}>();

defineEmits<{
  (e: 'back'): void;
}>();

const {
  activeRoomId,
  rooms,
  messages,
  members,
  messagesLoading,
  messagesMeta,
  sendMessage,
  deleteMessage,
  fetchMessages
} = useChat();

const authUser = useAuthUser();
const currentUserId = computed(() => authUser.value?.id || null);

const feedContainer = ref<HTMLElement | null>(null);
const inputText = ref('');
const isDetailsModalOpen = ref(false);

const currentRoom = computed(() => rooms.value.find(r => r.id === activeRoomId.value) || null);
const isDirect = computed(() => currentRoom.value?.room_type === 'direct');

const roomName = computed(() => {
  if (!currentRoom.value) return '';
  if (isDirect.value) {
    return currentRoom.value.direct_user
      ? formatUserName(currentRoom.value.direct_user)
      : 'Личный чат';
  }
  return currentRoom.value.name || 'Группа';
});

const avatarUrl = computed(() => {
  if (isDirect.value) return currentRoom.value?.direct_user?.avatar_url || null;
  return null;
});

// Роль текущего пользователя в комнате
const myRole = computed(() => {
  if (!currentUserId.value || isDirect.value) return 'member';
  return members.value.find(m => m.id === currentUserId.value)?.role || 'member';
});

const canDeleteForEveryone = (msg: IChatMessage) => {
  if (!currentUserId.value) return false;
  if (msg.sender_id === currentUserId.value) return true;
  if (!isDirect.value) {
    return myRole.value === 'owner' || myRole.value === 'moderator';
  }
  return false;
};

const shouldShowDateDivider = (msg: IChatMessage, index: number) => {
  if (index === 0) return true;
  const prevMsg = messages.value[index - 1];
  if (!prevMsg) return true;
  const prevDate = new Date(prevMsg.created_at * 1000).toDateString();
  const currDate = new Date(msg.created_at * 1000).toDateString();
  return prevDate !== currDate;
};

const isPaginating = ref(false);

const scrollToBottom = () => {
  requestAnimationFrame(() => {
    if (feedContainer.value) {
      feedContainer.value.scrollTop = feedContainer.value.scrollHeight;
    }
  });
};

onMounted(() => {
  scrollToBottom();
});

watch(activeRoomId, () => {
  nextTick(scrollToBottom);
});

/**
 * Реагирует на изменение длины массива сообщений.
 * Срабатывает при real-time WS-сообщениях и первой загрузке.
 * Во время пагинации (isPaginating=true) — пропускается.
 */
watch(
  () => messages.value.length,
  (newLen, oldLen) => {
    if (isPaginating.value) return;
    if (!feedContainer.value) return;

    const container = feedContainer.value;
    const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight;
    const isAtBottom = distanceFromBottom < 120;
    const lastMsg = messages.value[newLen - 1];
    const isMyMessage = lastMsg?.sender_id === currentUserId.value;
    const isFirstLoad = oldLen === 0;

    if (isAtBottom || isMyMessage || isFirstLoad) {
      nextTick(scrollToBottom);
    }
  }
);

const isLoadingMore = ref(false);

/**
 * Загружает предыдущую страницу сообщений.
 *
 * Стратегия «якорь»:
 * 1. Перед загрузкой берём data-msg-id первого видимого сообщения в DOM.
 * 2. Запускаем fetchMessages (новые элементы вставляются в начало массива).
 * 3. После обновления DOM находим якорный элемент и вызываем scrollIntoView.
 */
const loadMoreHistory = async () => {
  if (isLoadingMore.value || !feedContainer.value) return;
  if (!activeRoomId.value || !messagesMeta.value?.has_next) return;

  isLoadingMore.value = true;
  isPaginating.value = true;

  // Запоминаем первый [data-msg-id] элемент как якорь
  const container = feedContainer.value;
  const firstMsgEl = container.querySelector<HTMLElement>('[data-msg-id]');
  const anchorId = firstMsgEl?.dataset.msgId ?? null;

  const nextPage = messagesMeta.value.current_page + 1;
  await fetchMessages(activeRoomId.value, nextPage);

  // После обновления DOM — скроллим к якорю
  await nextTick();
  if (anchorId) {
    const anchor = container.querySelector<HTMLElement>(`[data-msg-id="${anchorId}"]`);
    anchor?.scrollIntoView({ block: 'start', behavior: 'instant' });
  }

  isLoadingMore.value = false;
  // Небольшая задержка перед сбросом флага —
  // watcher не должен вмешаться в тот же тик что и восстановление якоря
  setTimeout(() => { isPaginating.value = false; }, 50);
};

const handleScroll = () => {
  if (!feedContainer.value) return;
  // Триггер при приближении к верху (менее 60px)
  if (feedContainer.value.scrollTop < 60) {
    loadMoreHistory();
  }
};

const handleSendMessage = () => {
  if (!inputText.value.trim()) return;
  sendMessage(inputText.value);
  inputText.value = '';
};

// Enter — отправить, Shift+Enter — перенос строки (нативное поведение textarea)
const handleEnterKey = (e: KeyboardEvent) => {
  if (!e.shiftKey) {
    e.preventDefault();
    handleSendMessage();
  }
};
</script>

<style lang="scss" scoped>
.chat-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--ui-bg);
  min-height: 0;

  &__empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
    gap: 0.75rem;
  }

  &__empty-icon {
    font-size: 3.75rem;
    color: var(--ui-text-muted);
  }

  &__empty-title {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--ui-text-highlighted);
  }

  &__empty-text {
    font-size: 0.875rem;
    color: var(--ui-text-muted);
    max-width: 24rem;
  }

  &__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--ui-border);
    background-color: var(--ui-bg);
    flex-shrink: 0;
  }

  &__header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }

  &__back-btn {
    @media (min-width: 1024px) {
      display: none;
    }
  }

  &__avatar {
    background-color: var(--ui-primary-subtle);
    color: var(--ui-primary);
    font-weight: 600;

    &--direct {
      border-radius: 9999px;
    }

    &--group {
      border-radius: var(--ui-radius, 4px);
    }
  }

  &__header-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  &__header-name {
    font-weight: 700;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--ui-text-highlighted);
  }

  &__header-meta {
    font-size: 0.625rem;
    color: var(--ui-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  &__feed {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 1rem;
    display: flex;
    flex-direction: column;
    min-h: 0;
    -webkit-overflow-scrolling: touch;
    overflow-anchor: none;
  }

  &__loader {
    padding: 0.5rem 0;
    text-align: center;

    &--pagination {
      flex-shrink: 0;
    }
  }

  &__spinner {
    animation: spin 1s linear infinite;
    font-size: 1.125rem;
    color: var(--ui-text-muted);
  }

  &__no-messages {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    gap: 0.25rem;
  }

  &__date-divider {
    width: 100%;
    display: flex;
    justify-content: center;
    margin: 0.75rem 0;
    user-select: none;
  }

  &__date-text {
    padding: 0.25rem 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--ui-text-highlighted);
    background-color: var(--ui-bg-elevated);
    border: 1px solid var(--ui-border);
    border-radius: 0;
    letter-spacing: 0.02em;
  }

  &__input-bar {
    border-top: 1px solid var(--ui-border);
    padding: 0.75rem;
    background-color: var(--ui-bg);
    display: flex;
    align-items: stretch;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  &__textarea {
    flex: 1;
    font-size: 0.875rem;
    background-color: rgba(var(--ui-bg-muted), 0.2);
  }

  &__send-btn {
    flex-shrink: 0;
    height: auto;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
