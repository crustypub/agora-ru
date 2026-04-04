<template>
  <div class="container">
    <div class="auth-wrapper">
        <span class="auth-wrapper__title">Авторизация на ресурсе доступна исключительно через Telegram</span>
        <TelegramLoginWidget telegram-login="agoraru_auth_bot" @callback="testCallback" />
    </div>
  </div>
</template>

<script setup lang="ts">


const testCallback = (user: object) => {
  console.log("Custom callback function: ",user);
};
</script>

<style lang="scss" scoped>

.container {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
}

.auth-wrapper {
    width: 100%;
    max-width: 740px;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: .5rem;
}

</style>
