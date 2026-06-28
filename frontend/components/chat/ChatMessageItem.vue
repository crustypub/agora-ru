<template>
  <div 
    class="message-item"
    :class="isMe ? 'message-item--me' : 'message-item--other'"
  >
    <!-- Sender Avatar (Only in groups for other members) -->
    <UAvatar
      v-if="isGroup && !isMe"
      :src="message.author?.avatar_url || ''"
      :alt="senderName"
      size="xs"
      class="message-item__avatar"
    >
      <template v-if="!message.author?.avatar_url">
        {{ senderName.charAt(0).toUpperCase() }}
      </template>
    </UAvatar>

    <!-- Message bubble and content -->
    <div class="message-item__body">
      <!-- Sender Name (Only in groups for other members) -->
      <span 
        v-if="isGroup && !isMe" 
        class="message-item__sender-name"
      >
        {{ senderName }}
      </span>

      <!-- Bubble content -->
      <div 
        class="message-item__bubble group"
        :class="isMe ? 'message-item__bubble--me' : 'message-item__bubble--other'"
      >
        <!-- Dropdown Menu Actions -->
        <div class="message-item__actions">
          <UDropdownMenu :items="dropdownItems" :popper="{ placement: 'bottom-end' }">
            <UButton 
              color="neutral" 
              variant="ghost" 
              icon="material-symbols:more-vert" 
              size="xs"
              class="message-item__action-btn"
            />
          </UDropdownMenu>
        </div>

        <!-- Content -->
        <div class="message-item__text">
          {{ message.content }}
        </div>

        <!-- Timestamp & status info -->
        <div class="message-item__time-wrapper">
          <span>{{ formattedTime }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { IChatMessage } from '~/models/entities/chat.entities';
import { useNotify } from '~/composables/useNotify';
import { formatUserName, formatMessageTime } from '~/helpers/chat';

const props = defineProps<{
  message: IChatMessage;
  isMe: boolean;
  isGroup: boolean;
  canDeleteForEveryone: boolean;
}>();

const emit = defineEmits<{
  (e: 'delete', id: string, type: 'me' | 'everyone'): void;
}>();

const notify = useNotify();

// Determine sender display name
const senderName = computed(() => {
  return props.message.author
    ? formatUserName(props.message.author)
    : 'Пользователь';
});

// Format timestamp using shared utility
const formattedTime = computed(() => formatMessageTime(props.message.created_at));

// Dropdown actions setup
const dropdownItems = computed(() => {
  const items = [
    {
      label: 'Копировать',
      icon: 'material-symbols:content-copy-outline-rounded',
      onSelect: () => copyText(),
    },
    {
      label: 'Удалить у себя',
      icon: 'material-symbols:delete-outline-rounded',
      onSelect: () => emit('delete', props.message.id, 'me'),
    }
  ];

  if (props.canDeleteForEveryone) {
    items.push({
      label: 'Удалить для всех',
      icon: 'material-symbols:delete-forever-outline-rounded',
      onSelect: () => emit('delete', props.message.id, 'everyone'),
    });
  }

  return [items];
});

const copyText = async () => {
  try {
    await navigator.clipboard.writeText(props.message.content);
    notify.success('Скопировано в буфер обмена');
  } catch (err) {
    console.error('Failed to copy message:', err);
  }
};
</script>

<style lang="scss" scoped>
.message-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  max-width: 80%;
  margin: 0.375rem 0;

  &--me {
    align-self: flex-end;
    flex-direction: row-reverse;
  }

  &--other {
    align-self: flex-start;
    flex-direction: row;
  }

  &__avatar {
    margin-top: 0.25rem;
    flex-shrink: 0;
    border-radius: 9999px;
    background-color: var(--ui-primary-subtle);
    color: var(--ui-primary);
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  &__sender-name {
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--color-greek-blue-500);
    margin-left: 0.5rem;
  }

  &__bubble {
    position: relative;
    padding: 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    user-select: text;
    white-space: pre-wrap;
    word-break: break-word;
    min-width: 120px;
    max-width: 100%;

    &--me {
      border-bottom-right-radius: 0;
      background-color: var(--color-greek-blue-500);
      color: #fff;
    }

    &--other {
      background-color: var(--ui-bg-muted);
      color: var(--ui-text-highlighted);
      border-bottom-left-radius: 0;
      border: 1px solid var(--ui-border);
    }
  }

  &__actions {
    position: absolute;
    top: 0.375rem;
    right: 0.375rem;
    opacity: 0;
    transition: opacity 0.2s ease;

    .group:hover & {
      opacity: 1;
    }
  }

  &__action-btn {
    &:hover {
      background-color: rgba(0, 0, 0, 0.05);
      
      .dark & {
        background-color: rgba(255, 255, 255, 0.05);
      }
    }
  }

  &__text {
    padding-right: 1.5rem;
    font-weight: 400;
  }

  &__time-wrapper {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.25rem;
    margin-top: 0.25rem;
    font-size: 0.625rem;
    text-align: right;
    line-height: 1;
    user-select: none;

    .message-item__bubble--me & {
      color: rgba(255, 255, 255, 0.6);
    }

    .message-item__bubble--other & {
      color: var(--ui-text-muted);
    }
  }
}
</style>
