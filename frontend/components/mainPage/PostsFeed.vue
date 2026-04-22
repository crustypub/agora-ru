<template>
    <div class="posts-container">
        <div v-for="post in posts">
            <Post :data="post" />
        </div>
    </div>
</template>
<script setup lang="ts">
import { useApi } from '~/composables/useApi';
import type { IPostResponse } from '~/models/api/post.api';
import type { IPostResponseItem } from '~/models/entities/post.entities';
import Post from './Post.vue';

const posts = ref<IPostResponseItem[]>([]);

const { data: response } = await useApi<IPostResponse>('/api/post');

const setPosts = () => {
    if (response.value?.status === 'success' && Array.isArray(response.value.data)) {
        posts.value = response.value.data;
    }
};

setPosts();

watch(response, () => {
    setPosts();
});
</script>

<style lang="scss" scoped>
.posts-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
}
</style>