<template>
  <div 
    class="message-item"
    :class="isMe ? 'message-item--me' : 'message-item--other'"
  >
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

    <div class="message-item__body">
      <span 
        v-if="isGroup && !isMe" 
        class="message-item__sender-name"
      >
        {{ senderName }}
      </span>

      <div 
        class="message-item__bubble group"
        :class="isMe ? 'message-item__bubble--me' : 'message-item__bubble--other'"
      >
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

        <div class="message-item__text">
          {{ message.content }}
        </div>

        <div 
          v-if="message.attachments && message.attachments.length > 0" 
          class="message-item__attachments"
        >
          <a 
            v-for="att in message.attachments" 
            :key="att.id"
            :href="att.file_url"
            target="_blank"
            download
            class="message-item__attachment-link"
            :title="att.file_name"
          >
            <UIcon :name="getFileIcon(att.file_mime)" class="message-item__attachment-icon" />
            <div class="message-item__attachment-info">
              <span class="message-item__attachment-name">{{ att.file_name }}</span>
              <span class="message-item__attachment-size">{{ formatBytes(att.file_size) }}</span>
            </div>
          </a>
        </div>

        <div class="message-item__time-wrapper">
          <span>{{ formattedTime }}</span>
          <UIcon 
            v-if="isMe"
            :name="message.is_read ? 'material-symbols:done-all' : 'material-symbols:done'"
            class="message-item__status-icon"
          />
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
const getFileIcon = (mime: string) => {
  if (mime.startsWith('image/')) return 'material-symbols:image-outline';
  if (mime.startsWith('video/')) return 'material-symbols:video-library-outline';
  if (mime.startsWith('audio/')) return 'material-symbols:audiotrack-outline';
  return 'material-symbols:description-outline';
};

const formatBytes = (bytes: number, decimals = 2) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
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

  &__status-icon {
    font-size: 0.75rem;
    flex-shrink: 0;
    color: inherit;
  }

  &__attachments {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    margin-top: 0.5rem;
    margin-right: 1.5rem;
  }

  &__attachment-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border-radius: 4px;
    text-decoration: none;
    transition: background-color 0.2s ease;
    width: 100%;
    min-width: 0;

    .message-item__bubble--me & {
      background-color: rgba(255, 255, 255, 0.15);
      color: #fff;
      &:hover {
        background-color: rgba(255, 255, 255, 0.25);
      }
    }

    .message-item__bubble--other & {
      background-color: rgba(0, 0, 0, 0.05);
      color: var(--ui-text-highlighted);
      &:hover {
        background-color: rgba(0, 0, 0, 0.1);
      }
      .dark & {
        background-color: rgba(255, 255, 255, 0.05);
        &:hover {
          background-color: rgba(255, 255, 255, 0.1);
        }
      }
    }
  }

  &__attachment-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  &__attachment-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    line-height: 1.2;
  }

  &__attachment-name {
    font-size: 0.8125rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__attachment-size {
    font-size: 0.6875rem;
    opacity: 0.8;
  }
}
</style>
