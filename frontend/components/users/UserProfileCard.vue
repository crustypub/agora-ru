<script setup lang="ts">
import type { IUser } from '~/models/entities/user.entities';

interface Props {
  user: IUser | null;
  isOwnProfile: boolean;
}

const props = defineProps<Props>();
const { createDirectChat } = useChat();

const isWritingMessage = ref(false);

const handleWriteMessage = async () => {
  if (!props.user?.id || isWritingMessage.value) return;
  isWritingMessage.value = true;
  try {
    const res = await createDirectChat(props.user.id);
    if (res.success) {
      navigateTo('/chats');
    }
  } finally {
    isWritingMessage.value = false;
  }
};
</script>

<template>
  <UCard v-if="!user" class="user-profile-card__not-found">
    <div class="not-found-content">
      <UIcon name="material-symbols:person-off-outline-rounded" class="not-found-icon" />
      <h2 class="not-found-title">Пользователь не найден</h2>
      <p class="not-found-text">Пользователь с указанным идентификатором не существует или был удален.</p>
      <UButton color="primary" variant="soft" to="/users">К списку пользователей</UButton>
    </div>
  </UCard>

  <UCard v-else class="user-profile-card">
    <div class="user-profile">
      <div class="user-profile__header">
        <UAvatar
          :src="user.avatar_url || ''"
          :alt="user.first_name || user.username || '.'"
          size="3xl"
        />
        <div class="user-profile__info">
          <div class="user-profile__name-row">
            <h1 class="user-profile__name">
              {{ [user.first_name, user.last_name].filter(Boolean).join(' ') || user.username }}
            </h1>
            <UBadge v-if="isOwnProfile" color="primary" variant="subtle" size="sm">Это вы</UBadge>
          </div>
          <span v-if="user.username" class="user-profile__username">@{{ user.username }}</span>
          <p v-if="user.description" class="user-profile__description">{{ user.description }}</p>
        </div>
      </div>

      <div class="user-profile__actions">
        <UButton
          v-if="isOwnProfile"
          icon="material-symbols:settings-outline-rounded"
          color="neutral"
          variant="soft"
          to="/settings"
        >
          Настройки
        </UButton>

        <UButton
          v-else
          icon="material-symbols:chat-outline-rounded"
          color="primary"
          :loading="isWritingMessage"
          @click="handleWriteMessage"
        >
          Написать
        </UButton>
      </div>
    </div>
  </UCard>
</template>

<style lang="scss" scoped>
.user-profile-card {
  width: 100%;

  &__not-found {
    text-align: center;
    padding: 2rem 1rem;
    width: 100%;
  }
}

.not-found-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.not-found-icon {
  font-size: 3rem;
  color: $text-muted;
}

.not-found-title {
  font-size: $text-lg;
  font-weight: 600;
  color: $text-primary;
}

.not-found-text {
  font-size: $text-sm;
  color: $text-muted;
  margin-bottom: 0.5rem;
}

.user-profile {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;

  &__header {
    display: flex;
    align-items: center;
    gap: 1rem;

    @media (max-width: 640px) {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  &__name-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;

    @media (max-width: 640px) {
      justify-content: center;
    }
  }

  &__name {
    font-size: $text-xl;
    font-weight: 700;
    color: $text-primary;
    line-height: 1.2;
  }

  &__username {
    font-size: $text-sm;
    color: $text-muted;
  }

  &__description {
    font-size: $text-base;
    margin-top: 0.5rem;
    color: $text-secondary;
  }

  &__actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    border-top: 1px solid $border-color;
    padding-top: 1rem;

    @media (max-width: 640px) {
      justify-content: center;
    }
  }
}
</style>
