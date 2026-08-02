<template>
  <NuxtLink :to="`/users/${data.id}`" class="user-card-link">
    <UCard class="user-card">
      <div class="user-card__inner">
        <UAvatar :src="data.avatar_url || ''" :alt="data?.first_name || '.'" size="xl" />
        <div class="user-card__info">
          <span class="user-card__name">{{ [data.first_name, data.last_name].filter(Boolean).join(' ') || data.username }}</span>
          <span v-if="data?.username" class="user-card__username">@{{ data.username }}</span>
          <span v-if="data?.description" class="user-card__description">{{ data.description }}</span>
        </div>
      </div>
    </UCard>
  </NuxtLink>
</template>

<script setup lang="ts">
import type { IUser } from '~/models/entities/user.entities';

interface IProps {
  data: IUser
}

defineProps<IProps>();
</script>

<style lang="scss" scoped>
.user-card-link {
  display: block;
  text-decoration: none;
  color: inherit;
}

.user-card {
  width: 100%;
  min-height: 80px;
  height: max-content;
  cursor: pointer;

  &__inner {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  &__name {
    font-weight: 600;
    color: $text-primary;
  }

  &__username {
    font-size: $text-xs;
    color: $text-muted;
  }

  &__description {
    font-size: $text-sm;
    color: $text-secondary;
  }
}
</style>
