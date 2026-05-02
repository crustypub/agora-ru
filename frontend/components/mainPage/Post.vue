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
                        variant="ghost">{{ Number(postData?.comments_count) || 0 }}</UButton>
                    <UButton icon="material-symbols:remove" size="xs" color="red" :variant="postData?.is_disliked ? 'subtle' : 'ghost'"
                        @click="() => updatePostRating('Decrement', postData.is_disliked, postData.id)">{{
                            Number(postData?.rating_minus) || 0 }}</UButton>
                    <UButton icon="material-symbols:add" size="xs" color="green" :variant="postData?.is_liked ? 'subtle' : 'ghost'"
                        @click="() => updatePostRating('Increment', postData.is_liked, postData.id)">{{
                            Number(postData?.rating_plus) || 0 }}</UButton>
                </div>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { useApiCall } from '~/composables/useApi';
import { postFormatDateTime } from '~/helpers/common';
import type { IPostResponseItem, PostRatingUpdateMode } from '~/models/entities/post.entities';

interface IProps {
    data: IPostResponseItem
}

const { data } = defineProps<IProps>();

const postData = ref<IPostResponseItem>(data);

console.log('data', postData.value);

const updatePostRating = async function (mode: PostRatingUpdateMode, isClicked: boolean, postId: string) {
    const postDataSnapshot: IPostResponseItem = { ...postData.value };
    try {

        const requestData = {
            post_id: postId,
            mode: mode,
            operation_type: isClicked ? 'Remove' : 'Add'
        }
        if (mode === 'Increment') {
            postData.value = {
                ...postData.value,
                is_liked: !postData.value.is_liked,
                is_disliked: false,
                rating_plus: isClicked ? postData.value.rating_plus - 1 : postData.value.rating_plus + 1,
                rating_minus: postData.value.is_disliked ? postData.value.rating_minus - 1 : postData.value.rating_minus,
            }
        }

        else if (mode === 'Decrement') {
            postData.value = {
                ...postData.value,
                is_liked: false,
                is_disliked: !postData.value.is_disliked,
                rating_plus: postData.value.is_liked ? postData.value.rating_plus - 1 : postData.value.rating_plus,
                rating_minus: isClicked ? postData.value.rating_minus - 1 : postData.value.rating_minus + 1,
            }
        }

        await useApiCall('/api/post_rating', {
            method: 'POST',
            body: requestData,
        })
    } catch (e) {
        postData.value = postDataSnapshot
        console.error(e);
    }
}

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