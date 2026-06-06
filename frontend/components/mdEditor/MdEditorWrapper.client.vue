<template>
  <UForm :schema="schema" :state="state" class="md-editor-form" @submit="handleSubmit">

    <div class="md-editor-form__row">
      <UFormField label="Тип статьи" name="type" required class="md-editor-form__row-field">
        <USelect v-model="state.type" :items="typeOptions" value-attribute="value" option-attribute="label"
          placeholder="Выберите тип статьи" class="w-full" />
      </UFormField>

      <UFormField label="Заголовок" name="title" required class="md-editor-form__row-field">
        <UInput v-model="state.title" placeholder="Введите заголовок статьи" class="w-full" />
      </UFormField>
    </div>

    <UFormField label="Содержимое" name="content" required>
      <ClientOnly>
        <MdEditor v-model="state.content" language="en-US" :toolbars="activeToolbars" class="md-editor__wrapper"
          @on-upload-img="handleUploadImg" />
      </ClientOnly>
    </UFormField>

    <UButton type="submit" class="md-editor-form__submit">
      Сохранить
    </UButton>

  </UForm>
</template>

<script setup lang="ts">
import { MdEditor } from 'md-editor-v3';
import type { ToolbarNames } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import { date, z } from 'zod';
import { useApiCall } from '~/composables/useApi';
import { useNotify } from '~/composables/useNotify';
import type { IWikiType } from '~/models/entities/wiki.entities';

interface IProps {
  wikiTypes: IWikiType[] | undefined;
  closeModal: () => void;
  submit: () => void;
  article?: any; // Используем any или IWikiArticle
}

const { wikiTypes, closeModal, submit, article } = defineProps<IProps>();


const schema = z.object({
  type: z.number({ error: 'Выберите тип статьи' }).min(1, 'Выберите тип статьи'),
  title: z.string({ error: 'Введите заголовок' }).min(1, 'Введите заголовок'),
  content: z.string({ error: 'Введите содержимое' }).min(1, 'Введите содержимое'),
});

type ArticleFormState = z.output<typeof schema>;

const state = reactive({
  type: article?.wiki_type?.id ?? undefined as number | undefined,
  title: article?.title ?? '',
  content: article?.content ?? '',
});

const typeOptions = computed(() =>
  (wikiTypes ?? []).map((t) => ({ value: t.id, label: t.title }))
);

// На мобильных — только самые нужные кнопки, чтобы каждая была крупнее
const DESKTOP_TOOLBARS: ToolbarNames[] = [
  'preview', 'bold', 'underline', 'italic', 'strikeThrough', '-',
  'title', 'quote', 'unorderedList', 'orderedList', '-',
  'link', 'image', 'table', 'code', 'codeRow', '-',
  'revoke', 'next', '=',
  'pageFullscreen', 'fullscreen',
];

const MOBILE_TOOLBARS: ToolbarNames[] = [
  'preview', 'bold', 'italic', '-',
  'title', 'unorderedList', 'orderedList', '-',
  'link', 'code', '-',
  'revoke', '='
];

const isMobile = ref(false);
onMounted(() => {
  const mq = window.matchMedia('(max-width: 768px)');
  isMobile.value = mq.matches;
  mq.addEventListener('change', (e) => { isMobile.value = e.matches; });
});

const activeToolbars = computed(() =>
  isMobile.value ? MOBILE_TOOLBARS : DESKTOP_TOOLBARS
);

const handleUploadImg = async (
  files: File[],
  callback: (urls: string[]) => void
): Promise<void> => {
  const { error } = useNotify();

  const MAX_SIZE = 300 * 1024 * 1024; // 300 MB
  const hasOversizedFile = files.some(file => file.size > MAX_SIZE);
  if (hasOversizedFile) {
    error(
      'Ошибка загрузки',
      'Максимальный объем одного файла не должен превышать 300 МБ'
    );
    return;
  }

  const uploadPromises = files.map(async (file) => {
    try {
      const form = new FormData();
      form.append('file', file);
      const response = await useApiCall<{ url: string }>('/api/wiki/image', {
        method: 'POST',
        body: form,
      });
      return response.url;
    } catch (e: any) {
      console.error(`Failed to upload file ${file.name}:`, e);
      const serverError = e?.data?.error || `Не удалось загрузить файл "${file.name}"`;
      error(
        'Ошибка загрузки',
        serverError
      );
      return null;
    }
  });

  const results = await Promise.all(uploadPromises);
  const successfulUrls = results.filter((url): url is string => url !== null);

  callback(successfulUrls);
};

async function handleSubmit(event: { data: ArticleFormState }) {
  try {
    const requestData = {
      title: event?.data?.title,
      content: event?.data?.content,
      wiki_type_id: event?.data?.type,
    }
    
    if (article) {
      await useApiCall(`/api/wiki/${article.id}`, {
        method: 'PATCH',
        body: requestData,
      })
    } else {
      await useApiCall('/api/wiki', {
        method: 'POST',
        body: requestData,
      })
    }
    submit();
  } catch (e) {
    console.error('Failed to submit article:', e);
  }
  finally {
    closeModal();
  }
}
</script>

<style lang="scss" scoped>
.md-editor-form {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 1rem;
  height: 100%;

  &__row {
    display: flex;
    gap: 1rem;

    @media (max-width: 768px) {
      flex-direction: column;
    }
  }

  &__row-field {
    flex: 1;
    min-width: 0;
  }

  &__submit {
    align-self: flex-end;
  }
}

.md-editor__wrapper {
  flex: 1;

  // Увеличиваем зону касания кнопок тулбара на мобильных
  @media (max-width: 768px) {
    flex: none; // редактор не растягивается, кнопка идёт сразу после него

    :deep(.md-editor-toolbar-item) {
      min-width: 40px;
      min-height: 40px;
      padding: 0 10px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }

    :deep(.md-editor-toolbar) {
      padding: 6px 4px;
      gap: 2px;
    }
  }
}
</style>