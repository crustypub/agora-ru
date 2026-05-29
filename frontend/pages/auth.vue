<template>
  <div class="page-container items-center justify-center">
    <UCard class="auth-card">
      <template #header>
        <span class="auth-card__title">Авторизация</span>
      </template>
      <p class="auth-card__desc">Авторизация на ресурсе доступна исключительно через Telegram</p>
      <TelegramLoginWidget telegram-login="agoraru_auth_bot" @callback="testCallback" />
    </UCard>
  </div>
</template>

<script setup lang="ts">
import { useApi } from '~/composables/useApi';

const testCallback = async (user: object) => {
  try {
    const { data } = await useApi('/api/auth/telegram', {
      method: 'POST',
      body: user,
    })
    await navigateTo('/', { replace: true });
  } catch(e) {
    console.error(e);
  }
};
</script>

<style lang="scss" scoped>
.auth-card {
  width: 100%;
  max-width: 740px;

  &__title {
    font-size: $text-xl;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__desc {
    font-size: $text-sm;
    color: var(--ui-text-muted);
    margin-bottom: 1rem;
  }
}
</style>
