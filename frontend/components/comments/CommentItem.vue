<template>
    <div class="comment-item">
        <div class="comment-item__header">
            <div class="comment-item__author">
                <UAvatar :src="comment.author.avatar_url || ''" :alt="comment.author?.first_name || '.'" size="sm" />
                <div class="comment-item__author-info">
                    <span class="comment-item__name">{{ comment.author.first_name || comment.author.username }}</span>
                    <span class="comment-item__date">{{ formattedDate }}</span>
                </div>
            </div>

            <UDropdownMenu v-if="isOwner" :items="dropdownItems" :popper="{ placement: 'bottom-end' }">
                <UButton color="primary" variant="ghost" icon="i-heroicons-ellipsis-horizontal-20-solid" />
            </UDropdownMenu>
        </div>

        <div v-if="isEditing" class="comment-item__edit-form">
            <UTextarea v-model="editContent" autoresize />
            <div class="comment-item__actions">
                <UButton size="sm" color="primary" variant="soft" @click="cancelEdit">Отмена</UButton>
                <UButton size="sm" color="primary" @click="saveEdit" :loading="isSaving">Сохранить</UButton>
            </div>
        </div>
        <div v-else class="comment-item__content">
            {{ comment.content }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { IComment } from '~/models/entities/comment.entities';
import { postFormatDateTime } from '~/helpers/common';

interface IProps {
    comment: IComment;
    currentUserId: string | null;
}

const props = defineProps<IProps>();
const emit = defineEmits<{
    (e: 'update', id: string, newContent: string): void;
    (e: 'delete', id: string): void;
}>();

const isOwner = computed(() => props.currentUserId && props.comment.author && props.currentUserId === props.comment.author.id);

const isEditing = ref(false);
const editContent = ref('');
const isSaving = ref(false);

const formattedDate = computed(() => {
    return postFormatDateTime(props.comment.created_at);
});

const dropdownItems = computed(() => [[
    {
        label: 'Редактировать',
        icon: 'i-heroicons-pencil-square-20-solid',
        onSelect: () => startEdit()
    },
    {
        label: 'Удалить',
        icon: 'i-heroicons-trash-20-solid',
        onSelect: () => emit('delete', props.comment.id)
    }
]]);


const startEdit = () => {
    editContent.value = props.comment.content;
    isEditing.value = true;
};

const cancelEdit = () => {
    isEditing.value = false;
    editContent.value = '';
};

const saveEdit = async () => {
    if (!editContent.value.trim()) return;
    isSaving.value = true;
    try {
        await emit('update', props.comment.id, editContent.value);
        isEditing.value = false;
    } finally {
        isSaving.value = false;
    }
};
</script>

<style lang="scss" scoped>
.comment-item {
    display: flex;
    flex-direction: column;
    padding: 1rem;
    background-color: $bg-primary;
    border: 1px solid $border-color;
    border-radius: 8px;
    gap: 0.75rem;

    &__header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    &__author {
        display: flex;
        gap: 0.75rem;
        align-items: center;
    }

    &__author-info {
        display: flex;
        flex-direction: column;
    }

    &__name {
        font-weight: 600;
        font-size: $text-sm;
        color: $text-primary;
    }

    &__date {
        font-size: $text-xs;
        color: $text-muted;
    }

    &__content {
        font-size: $text-sm;
        color: $text-primary;
        white-space: pre-wrap;
        word-break: break-word;
    }

    &__edit-form {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    &__actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
    }
}
</style>
