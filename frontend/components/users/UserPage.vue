<script setup lang="ts">
import type { IUser } from '~/models/entities/user.entities';
import UserProfileCard from './UserProfileCard.vue';

interface Props {
  user: IUser | null;
  isOwnProfile?: boolean;
}

defineProps<Props>();
</script>

<template>
  <div class="user-page">
    <UserProfileCard
      :user="user"
      :is-own-profile="!!isOwnProfile"
    />
  </div>
</template>

<style lang="scss" scoped>
.user-page {
  flex: 1;
  width: 100%;
}
</style>
