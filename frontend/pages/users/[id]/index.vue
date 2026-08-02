<script setup lang="ts">
import UserPage from '~/components/users/UserPage.vue';
import type { IUser } from '~/models/entities/user.entities';

const route = useRoute();
const id = route.params.id as string;
const authUser = useAuthUser();

const isOwn = computed(() => id === authUser.value?.id);
const { data: response } = !isOwn.value ? await useApi<{ data: IUser }>(`/api/users/id/${id}`) : { data: ref(null) };
const user = computed(() => isOwn.value ? authUser.value : response.value?.data || null);

useHead({
  title: computed(() => user.value
    ? `Agora RU — ${[user.value.first_name, user.value.last_name].filter(Boolean).join(' ') || user.value.username}`
    : 'Agora RU — Пользователь не найден')
});
</script>

<template>
  <div class="page-container">
    <UserPage :user="user" :is-own-profile="isOwn" />
  </div>
</template>
