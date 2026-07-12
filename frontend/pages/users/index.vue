<template>
  <div class="page-container">
    <UserListPage />
  </div>
</template>

<script setup lang="ts">
import UserListPage from '~/components/users/UserListPage.vue';

useHead({
  title: 'Agora RU — Пользователи',
})
</script>
