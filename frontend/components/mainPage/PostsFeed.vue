<template>
    <div v-for="post in posts">
        {{ post.id }}
    </div>
</template>
<script setup lang="ts">
import { useApi } from '~/composables/useApi';
import type { IPostResponse } from '~/models/api/post.api';
import type { IPostResponseItem } from '~/models/entities/post.entities';

const posts = ref<IPostResponseItem[]>([]);

// key уникален для этого компонента, чтобы Nuxt не переиспользовал
// кешированный результат от другого вызова useFetch с тем же URL
const { data: response } = await useApi<IPostResponse>('/api/post');

const setPosts = () => {
    if (response.value?.status === 'success' && Array.isArray(response.value.data)) {
        posts.value = response.value.data;
    }
};

// Вызываем сразу — на случай если данные уже пришли по SSR
setPosts();

// Следим за изменениями — для клиентской загрузки
watch(response, () => {
    console.log('response', response)
    setPosts();
});

onMounted(() => {
    console.log('response2', response.value)
})
</script>

<style lang="scss" scoped>

</style>