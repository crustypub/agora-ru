<template>
    <div class="comment-section" id="comments">
        <h3 class="comment-section__title">Комментарии</h3>
        
        <div class="comment-section__form" v-if="currentUserId">
            <UTextarea 
                v-model="newCommentText" 
                placeholder="Написать комментарий..." 
                autoresize
                :disabled="isSubmitting"
            />
            <div class="comment-section__form-actions">
                <UButton 
                    color="primary" 
                    @click="submitComment" 
                    :loading="isSubmitting"
                    :disabled="!newCommentText.trim()"
                >
                    Отправить
                </UButton>
            </div>
        </div>
        <div v-else class="comment-section__auth-prompt">
            Войдите, чтобы оставить комментарий.
        </div>

        <div class="comment-section__list">
            <CommentItem 
                v-for="comment in comments" 
                :key="comment.id"
                :comment="comment"
                :current-user-id="currentUserId"
                @update="updateComment"
                @delete="deleteComment"
            />
            <div v-if="comments.length === 0" class="comment-section__empty">
                Комментариев пока нет. Будьте первыми!
            </div>
        </div>
        
        <UModal v-model:open="isDeleteModalOpen">
            <template #content>
                <div class="p-6">
                    <h3 class="text-lg font-semibold mb-2">Удаление комментария</h3>
                    <p class="text-sm text-gray-500 mb-6">Вы уверены, что хотите удалить этот комментарий? Это действие нельзя отменить.</p>
                    <div class="flex justify-end gap-3">
                        <UButton color="gray" variant="soft" @click="isDeleteModalOpen = false">Отмена</UButton>
                        <UButton color="red" @click="confirmDelete" :loading="isDeleting">Удалить</UButton>
                    </div>
                </div>
            </template>
        </UModal>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAuthUser } from '~/composables/useAuthUser';
import { useApiCall } from '~/composables/useApi';
import type { IComment } from '~/models/entities/comment.entities';
import CommentItem from './CommentItem.vue';

interface IProps {
    entityType: string;
    entityId: string;
}

const props = defineProps<IProps>();

const { data: commentsResponse } = await useApi<{ data: IComment[] }>(`/api/comments`, {
    query: {
        entity_type: props.entityType,
        entity_id: props.entityId,
        limit: 50,
    }
});

const comments = ref<IComment[]>(commentsResponse.value?.data || []);
const newCommentText = ref('');
const isSubmitting = ref(false);

const isDeleteModalOpen = ref(false);
const commentToDelete = ref<string | null>(null);
const isDeleting = ref(false);

const authUser = useAuthUser();
const currentUserId = computed(() => authUser.value?.id || null);

const submitComment = async () => {
    if (!newCommentText.value.trim()) return;
    
    isSubmitting.value = true;
    try {
        const response = await useApiCall<{ data: IComment }>('/api/comments', {
            method: 'POST',
            body: {
                entity_type: props.entityType,
                entity_id: props.entityId,
                content: newCommentText.value
            }
        });
        
        if (response && response.data) {
            // Если бэкенд возвращает только id автора (String), подставляем профиль текущего пользователя
            const createdComment = response.data;
            if (typeof createdComment.author === 'string' && authUser.value) {
                createdComment.author = authUser.value;
            }
            comments.value.unshift(createdComment);
            newCommentText.value = '';
        }
    } catch (e) {
        console.error('Failed to submit comment', e);
    } finally {
        isSubmitting.value = false;
    }
};

const updateComment = async (id: string, newContent: string) => {
    try {
        const response = await useApiCall<{ data: IComment }>(`/api/comments/${id}`, {
            method: 'PATCH',
            body: {
                content: newContent
            }
        });
        
        if (response && response.data) {
            const index = comments.value.findIndex(c => c.id === id);
            if (index !== -1) {
                const updatedComment = response.data;
                if (typeof updatedComment.author === 'string' && authUser.value) {
                    updatedComment.author = authUser.value;
                }
                comments.value[index] = updatedComment;
            }
        }
    } catch (e) {
        console.error('Failed to update comment', e);
        throw e;
    }
};

const deleteComment = (id: string) => {
    commentToDelete.value = id;
    isDeleteModalOpen.value = true;
};

const confirmDelete = async () => {
    if (!commentToDelete.value) return;
    const id = commentToDelete.value;
    isDeleting.value = true;
    
    try {
        await useApiCall(`/api/comments/${id}`, {
            method: 'DELETE'
        });
        comments.value = comments.value.filter(c => c.id !== id);
        isDeleteModalOpen.value = false;
        commentToDelete.value = null;
    } catch (e) {
        console.error('Failed to delete comment', e);
    } finally {
        isDeleting.value = false;
    }
};
</script>

<style lang="scss" scoped>
.comment-section {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    width: 100%;
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 1px solid $border-color;

    &__title {
        font-size: $text-xl;
        font-weight: 600;
        color: $text-primary;
    }

    &__form {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background-color: $bg-primary;
        padding: 1rem;
        border-radius: 8px;
        border: 1px solid $border-color;
    }

    &__form-actions {
        display: flex;
        justify-content: flex-end;
    }

    &__auth-prompt {
        padding: 1rem;
        background-color: $bg-secondary;
        border-radius: 8px;
        text-align: center;
        color: $text-muted;
        font-size: $text-sm;
    }

    &__list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    &__empty {
        text-align: center;
        color: $text-muted;
        font-size: $text-sm;
        padding: 2rem 0;
    }
}
</style>
