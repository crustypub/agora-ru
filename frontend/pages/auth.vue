<template>
  <div class="page-container items-center">
    <div class="auth-wrapper">
      <span class="auth-wrapper__title">Авторизация на ресурсе доступна исключительно через Telegram</span>
      <TelegramLoginWidget telegram-login="agoraru_auth_bot" @callback="testCallback" />
    </div>
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
.auth-wrapper {
  width: 100%;
  max-width: 740px;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: .5rem;
}
</style>
