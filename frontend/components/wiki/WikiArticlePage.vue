<template>
  <div class="wiki-article-container">
    <!-- Кнопка назад -->
    <div class="wiki-article__back">
      <UButton
        icon="material-symbols:arrow-back-rounded"
        variant="ghost"
        color="neutral"
        size="sm"
        @click="navigateTo('/wiki')"
      >
        Назад к списку
      </UButton>
    </div>

    <!-- Загрузка / Ошибка -->
    <div v-if="!response?.data" class="wiki-article__loading">
      <USkeleton class="h-8 w-3/4 mb-4" />
      <USkeleton class="h-4 w-1/2 mb-2" />
      <USkeleton class="h-4 w-1/3 mb-8" />
      <USkeleton class="h-40 w-full" />
    </div>

    <template v-else>
      <!-- Шапка статьи -->
      <header class="wiki-article__header">
        <div class="wiki-article__meta-top">
          <UBadge color="primary" variant="subtle" size="md">
            {{ response.data.wiki_type.title }}
          </UBadge>
          <div class="wiki-article__status">
            <span v-if="response.data.is_confirmed" class="status-badge status-badge--confirmed">
              <UIcon name="material-symbols:check-circle-rounded" class="status-badge__icon" />
              Подтверждено
            </span>
            <span v-else class="status-badge status-badge--pending">
              <UIcon name="material-symbols:info-outline-rounded" class="status-badge__icon" />
              На проверке
            </span>
          </div>
        </div>

        <div class="wiki-article__title-row">
          <h1 class="wiki-article__title">{{ response.data.title }}</h1>
          
          <!-- Кнопки управления (только для автора) -->
          <div v-if="isAuthor" class="wiki-article__actions">
            <UButton
              icon="material-symbols:edit-outline-rounded"
              color="primary"
              variant="soft"
              size="sm"
              @click="isEditModalOpen = true"
            >
              Редактировать
            </UButton>
            <UButton
              icon="material-symbols:delete-outline-rounded"
              color="danger"
              variant="soft"
              size="sm"
              @click="confirmDelete"
            >
              Удалить
            </UButton>
          </div>
        </div>

        <!-- Информация об авторах -->
        <div class="wiki-article__authors">
          <div class="author-card">
            <span class="author-card__label">Автор:</span>
            <div class="author-card__user">
              <UAvatar
                :src="response.data.created_by.avatar_url || ''"
                :alt="response.data.created_by.first_name || '.'"
                size="sm"
              />
              <div class="author-card__details">
                <span class="author-card__name">
                  {{ response.data.created_by.first_name || response.data.created_by.username }}
                </span>
                <span class="author-card__date">{{ formatDate(response.data.created_at) }}</span>
              </div>
            </div>
          </div>

          <div 
            v-if="response.data.last_edited_by && response.data.updated_at !== response.data.created_at" 
            class="author-card"
          >
            <span class="author-card__label">Редактор:</span>
            <div class="author-card__user">
              <UAvatar
                :src="response.data.last_edited_by.avatar_url || ''"
                :alt="response.data.last_edited_by.first_name || '.'"
                size="sm"
              />
              <div class="author-card__details">
                <span class="author-card__name">
                  {{ response.data.last_edited_by.first_name || response.data.last_edited_by.username }}
                </span>
                <span class="author-card__date">{{ formatDate(response.data.updated_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </header>

      <!-- Тело статьи -->
      <article class="wiki-article__content">
        <MdPreview editorId="preview-only" :modelValue="response.data.content" language="en-US" />
      </article>

      <!-- Комментарии -->
      <section class="wiki-article__comments">
        <CommentSection v-if="articleIdStr" entity-type="wiki" :entity-id="articleIdStr" />
      </section>
    </template>

    <!-- Модалка редактирования -->
    <WikiArticleModal
      v-if="isAuthor"
      v-model="isEditModalOpen"
      :wiki-types="wikiTypesResponse?.data"
      :article="response?.data"
      :submit="handleEditSubmit"
    />

    <!-- Диалог подтверждения удаления -->
    <UModal v-model:open="isDeleteModalOpen" title="Удаление статьи">
      <template #body>
        <p class="delete-confirm-text">Вы уверены, что хотите безвозвратно удалить эту статью?</p>
      </template>
      <template #footer>
        <div class="delete-confirm-actions">
          <UButton color="neutral" variant="ghost" @click="isDeleteModalOpen = false">Отмена</UButton>
          <UButton color="danger" @click="handleDeleteSubmit" :loading="isDeleting">Удалить</UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { MdPreview } from 'md-editor-v3';
import type { IWikiArticleResponse, IWikiTypesResponse } from '~/models/api/wiki.api';
import CommentSection from '../comments/CommentSection.vue';
import WikiArticleModal from './WikiArticleModal.vue';
import { useApi, useApiCall } from '~/composables/useApi';

interface IProps {
  articleId: string | string[] | undefined;
}

const props = defineProps<IProps>();

const articleIdStr = computed(() => {
  if (Array.isArray(props.articleId)) return props.articleId[0];
  return props.articleId as string;
});

// Запрос статьи
const { data: response, refresh } = await useApi<IWikiArticleResponse>(`/api/wiki/${articleIdStr.value}`);

// Запрос типов (нужен для формы редактирования)
const { data: wikiTypesResponse } = await useApi<IWikiTypesResponse>('/api/wiki_types');

// Авторизованный пользователь
const authUser = useAuthUser();

// Проверка на авторство
const isAuthor = computed(() => {
  if (!authUser.value || !response.value?.data) return false;
  return response.value.data.created_by.id === authUser.value.id;
});

// Состояние UI
const isEditModalOpen = ref(false);
const isDeleteModalOpen = ref(false);
const isDeleting = ref(false);

const formatDate = (timestamp: number) => {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};

const handleEditSubmit = () => {
  refresh();
};

const confirmDelete = () => {
  isDeleteModalOpen.value = true;
};

const handleDeleteSubmit = async () => {
  isDeleting.value = true;
  try {
    await useApiCall(`/api/wiki/${articleIdStr.value}`, {
      method: 'DELETE'
    });
    isDeleteModalOpen.value = false;
    navigateTo('/wiki');
  } catch (e) {
    console.error('Failed to delete wiki article:', e);
  } finally {
    isDeleting.value = false;
  }
};
</script>

<style lang="scss" scoped>
.wiki-article-container {
  width: 100%;
  max-width: 900px;
  margin: 0 auto;
  padding: 1rem 0 4rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.wiki-article {
  &__back {
    align-self: flex-start;
  }

  &__loading {
    background-color: $bg-primary;
    border: 1px solid $border-color;
    border-radius: 12px;
    padding: 2rem;
  }

  &__header {
    background-color: $bg-primary;
    border: 1px solid $border-color;
    border-radius: 12px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: 0 2px 8px rgba($black, 0.01);
  }

  &__meta-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  &__title-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1.5rem;

    @media (max-width: 768px) {
      flex-direction: column;
      align-items: stretch;
    }
  }

  &__title {
    font-size: 1.85rem;
    font-weight: 700;
    color: $text-primary;
    line-height: 1.3;
    margin: 0;
  }

  &__actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;

    @media (max-width: 768px) {
      justify-content: flex-start;
    }
  }

  &__authors {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid $border-color;
  }

  &__content {
    background-color: $bg-primary;
    border: 1px solid $border-color;
    border-radius: 12px;
    padding: 1.5rem;
    min-height: 200px;
    box-shadow: 0 2px 8px rgba($black, 0.01);
  }
}

.author-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;

  &__label {
    font-size: $text-xs;
    color: $text-muted;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.05em;
  }

  &__user {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  &__details {
    display: flex;
    flex-direction: column;
  }

  &__name {
    font-size: $text-sm;
    font-weight: 600;
    color: $text-primary;
  }

  &__date {
    font-size: $text-xs;
    color: $text-muted;
  }
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: $text-xs;
  font-weight: 600;
  padding: 0.25rem 0.6rem;
  border-radius: 9999px;

  &--confirmed {
    color: #10b981;
    background-color: rgba(#10b981, 0.08);
  }

  &--pending {
    color: #f59e0b;
    background-color: rgba(#f59e0b, 0.08);
  }

  &__icon {
    font-size: 1rem;
  }
}

.delete-confirm-text {
  font-size: $text-sm;
  color: $text-secondary;
}

.delete-confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  width: 100%;
}

@media (max-width: 768px) {
  .wiki-article-container {
    padding: 1rem 1rem 4rem;
  }
  .wiki-article {
    &__header, &__content {
      padding: 1rem;
    }
  }
}
</style>