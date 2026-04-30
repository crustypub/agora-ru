<template>
    <div class="post-container">
        <UAvatar :src="data?.author?.avatar_url || ''" :alt="data?.author?.first_name || '.'" size="md" />
        <div class="post-content">
            <span class="post-content__title">
                {{ data.title }}
            </span>
            <div class="post-footer">
                <div class="post-footer__user">
                    <span class="post-footer__name" v-if="data?.author?.first_name">{{ data.author.first_name }}</span>
                    <NuxtLink to="/" class="post-footer__username" v-if="data?.author?.username">@{{
                        data.author.username }}
                    </NuxtLink>
                    <div class="post-footer__separator" />
                    <ClientOnly>
                        <span class="post-footer__created_at">{{ postFormatDateTime(data.created_at) }}</span>
                    </ClientOnly>
                </div>
                <div class="post-footer__action-btns">
                    <UButton 
                        icon="material-symbols:mode-comment-outline-rounded"
                        size="xs"
                        color="primary"
                        variant="subtle"
                    >{{ Number(data?.comments_count) || 0}}</UButton>
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
    min-width: 100%;
    min-height: 2rem;
    height: max-content;
    display: flex;
    padding: .5rem;
    background-color: $white-off;
    align-items: center;
    column-gap: .5rem;
}

.post-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;

    &__title {
        color: $gray-800;
        font-size: $text-md;
        font-weight: 600;
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

    &__username {
        color: $blue-light;
    }

    &__separator {
        width: 3px;
        height: 3px;
        border-radius: 50%;
        background-color: $gray-800;
    }

    &__created_at {
        font-size: $text-xs;
        color: $gray-600;
    }
}
</style>