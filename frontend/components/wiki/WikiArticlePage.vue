<template>
  <div class="wiki-article-container">
    <!-- Загрузка / Ошибка -->
    <UCard v-if="!response?.data">
      <USkeleton class="h-8 w-3/4 mb-4" />
      <USkeleton class="h-4 w-1/2 mb-2" />
      <USkeleton class="h-4 w-1/3 mb-8" />
      <USkeleton class="h-40 w-full" />
    </UCard>

    <template v-else>
      <!-- Шапка статьи -->
      <UCard>
        <template #header>
          <div class="wiki-article__meta-top">
            <UBadge color="primary" variant="subtle" size="md">
              {{ response.data.wiki_type.title }}
            </UBadge>
            <UBadge
              v-if="response.data.is_confirmed"
              color="success"
              variant="subtle"
              size="sm"
            >
              <UIcon name="material-symbols:check-circle-rounded" class="size-4" />
              Подтверждено
            </UBadge>
            <UBadge
              v-else
              color="warning"
              variant="subtle"
              size="sm"
            >
              <UIcon name="material-symbols:info-outline-rounded" class="size-4" />
              На проверке
            </UBadge>
          </div>
        </template>

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
              color="error"
              variant="soft"
              size="sm"
              @click="confirmDelete"
            >
              Удалить
            </UButton>
          </div>
        </div>

        <template #footer>
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
        </template>
      </UCard>

      <!-- Тело статьи -->
      <UCard>
        <MdPreview editorId="preview-only" :modelValue="response.data.content" language="en-US" />
      </UCard>

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
          <UButton color="error" @click="handleDeleteSubmit" :loading="isDeleting">Удалить</UButton>
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
const router = useRouter();

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

const goBack = () => {
  router.back();
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
  padding: 1rem 0 4rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.wiki-article {
  &__back {
    align-self: flex-start;
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
    color: var(--ui-text-highlighted);
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
  }
}

.author-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;

  &__label {
    font-size: $text-xs;
    color: var(--ui-text-muted);
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
    color: var(--ui-text-highlighted);
  }

  &__date {
    font-size: $text-xs;
    color: var(--ui-text-muted);
  }
}

.delete-confirm-text {
  font-size: $text-sm;
  color: var(--ui-text-muted);
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
}
</style>