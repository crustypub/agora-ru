<template>
  <div
    class="room-item"
    :class="{ 'room-item--active': isActive }"
    @click="$emit('select')"
  >
    <!-- Room Avatar -->
    <UAvatar
      :src="avatarUrl || ''"
      :alt="roomName"
      :class="isDirect ? 'room-item__avatar--direct' : 'room-item__avatar--group'"
      size="md"
      class="room-item__avatar"
    >
      <template v-if="!avatarUrl">
        {{ roomName.charAt(0).toUpperCase() }}
      </template>
    </UAvatar>

    <!-- Room Text Info -->
    <div class="room-item__info">
      <div class="room-item__header">
        <span class="room-item__name">
          {{ roomName }}
        </span>
        <span v-if="formattedTime" class="room-item__time">
          {{ formattedTime }}
        </span>
      </div>

      <div class="room-item__footer">
        <span class="room-item__last-message">
          <span v-if="isLastMessageMine" class="room-item__mine-prefix">Вы:</span>
          {{ lastMessageText }}
        </span>
        <span 
          v-if="room.unread_count > 0" 
          class="room-item__unread"
        >
          {{ room.unread_count }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { IChatListItem } from '~/models/entities/chat.entities';
import { useAuthUser } from '~/composables/useAuthUser';
import { formatUserName, formatRoomTime, stripLinkPreview } from '~/helpers/chat';

const props = defineProps<{
  room: IChatListItem;
  isActive: boolean;
}>();

defineEmits<{
  (e: 'select'): void;
}>();

const authUser = useAuthUser();

const isDirect = computed(() => props.room.room_type === 'direct');

// Determine room name
const roomName = computed(() => {
  if (isDirect.value) {
    return props.room.direct_user
      ? formatUserName(props.room.direct_user)
      : 'Личный чат';
  }
  return props.room.name || 'Групповой чат';
});

// Determine room avatar
const avatarUrl = computed(() => {
  if (isDirect.value) {
    return props.room.direct_user?.avatar_url || null;
  }
  return null; // Group chats get letter placeholder
});

// Last message text preview
const lastMessageText = computed(() => {
  if (!props.room.last_message) return 'Сообщений нет';
  return stripLinkPreview(props.room.last_message.content);
});

// Check if last message is sent by current user
const isLastMessageMine = computed(() => {
  if (!props.room.last_message || !authUser.value) return false;
  return props.room.last_message.sender_id === authUser.value.id;
});

// Format timestamp using shared utility
const formattedTime = computed(() => {
  const timestamp = props.room.last_message?.created_at || props.room.updated_at;
  return formatRoomTime(timestamp);
});
</script>

<style lang="scss" scoped>
.room-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  cursor: pointer;
  transition: background-color 0.2s ease;
  border-bottom: 1px solid var(--ui-border);
  
  &:hover {
    background-color: var(--ui-bg-muted);
  }

  &:last-child {
    border-bottom: none;
  }

  &--active {
    background-color: var(--ui-bg-muted);
    border-left: 3px solid var(--ui-primary);
  }

  &__avatar {
    background-color: var(--ui-primary-subtle);
    color: var(--ui-primary);

    &--direct {
      border-radius: 9999px;
    }

    &--group {
      border-radius: var(--ui-radius, 4px);
    }
  }

  &__info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  &__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  &__name {
    font-weight: 600;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--ui-text-highlighted);
  }

  &__time {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    white-space: nowrap;
    margin-left: 0.5rem;
  }

  &__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  &__last-message {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    margin-right: 0.5rem;
  }

  &__mine-prefix {
    font-weight: 500;
    color: var(--ui-text-highlighted);
    margin-right: 0.25rem;
  }

  &__unread {
    padding: 0.125rem 0.375rem;
    font-size: 0.625rem;
    font-weight: 700;
    color: #fff;
    background-color: var(--ui-primary);
    border-radius: 9999px;
    flex-shrink: 0;
    min-width: 1.25rem;
    text-align: center;
    line-height: 1;
  }
}
</style>
