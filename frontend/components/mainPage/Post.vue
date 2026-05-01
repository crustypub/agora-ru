<template>
    <div class="post-container">
        <UAvatar :src="data?.author?.avatar_url || ''" :alt="data?.author?.first_name || '.'" size="md" />
        <div class="post-content">
            <div class="post-header">
                <span class="post-content__title">
                    {{ data.title }}
                </span>
                <div class="post-header__time">
                    <ClientOnly>
                        <span class="post-footer__created_at">{{ postFormatDateTime(data.created_at) }}</span>
                    </ClientOnly>
                </div>
            </div>
            <div class="post-footer">
                <div class="post-footer__user">
                    <span class="post-footer__name" v-if="data?.author?.first_name">{{ data.author.first_name }}</span>
                    <NuxtLink to="/" class="post-footer__username" v-if="data?.author?.username">@{{
                        data.author.username }}
                    </NuxtLink>
                </div>
                <div class="post-footer__action-btns">
                    <UButton icon="material-symbols:mode-comment-outline-rounded" size="xs" color="neutral"
                        variant="subtle">{{ Number(data?.comments_count) || 0 }}</UButton>
                    <UButton icon="material-symbols:remove" size="xs" color="error" variant="subtle">{{
                        Number(data?.comments_count) || 0 }}</UButton>
                    <UButton icon="material-symbols:add" size="xs" color="secondary" variant="subtle">{{
                        Number(data?.comments_count) || 0 }}</UButton>
                </div>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { postFormatDateTime } from '~/helpers/common';
import type { IPostResponseItem } from '~/models/entities/post.entities';

interface IProps {
    data: IPostResponseItem
}

const { data } = defineProps<IProps>();

console.log('data', data);

</script>

<style lang="scss" scoped>
.post-container {
    width: 100%;
    min-height: 2rem;
    height: max-content;
    display: flex;
    padding: .5rem;
    background-color: $white-off;
    align-items: center;
    column-gap: .5rem;
    font-size: 11px;
}

.post-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-width: 0;

    &__title {
        color: $gray-800;
        font-size: $text-base;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: block;
    }
}

.post-header {
    width: 100%;
    height: max-content;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    overflow: hidden;
    row-gap: 4px;
    column-gap: 8px;

    &__time {
        width: max-content;
        display: flex;
        justify-content: flex-end;
        flex-shrink: 0;
    }
}

.post-footer {
    width: 100%;
    justify-content: space-between;
    display: flex;
    align-items: center;

    &__user {
        display: flex;
        align-items: center;
        column-gap: 4px;
    }

    &__name {
        font-size: 1em;
    }

    &__username {
        color: $blue-light;
    }

    &__created_at {
        font-size: $text-xs;
        color: $gray-600;
        white-space: nowrap;
    }

    &__action-btns {
        display: flex;
        column-gap: 8px;
    }
}
</style>