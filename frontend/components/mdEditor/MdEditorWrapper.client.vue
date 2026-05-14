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
        <MdEditor v-model="state.content" language="en-US" :toolbars="activeToolbars" class="md-editor__wrapper" />
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
import type { IWikiTypeResponseItem } from '~/models/entities/wiki.entities';

interface IProps {
  wikiTypes: IWikiTypeResponseItem[] | undefined;
  closeModal: () => void;
}

const { wikiTypes, closeModal } = defineProps<IProps>();


const schema = z.object({
  type: z.number({ error: 'Выберите тип статьи' }).min(1, 'Выберите тип статьи'),
  title: z.string({ error: 'Введите заголовок' }).min(1, 'Введите заголовок'),
  content: z.string({ error: 'Введите содержимое' }).min(1, 'Введите содержимое'),
});

type ArticleFormState = z.output<typeof schema>;

const state = reactive({
  type: undefined as number | undefined,
  title: '',
  content: '',
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

async function handleSubmit(event: { data: ArticleFormState }) {
  try {
    const requestData = {
      title: event?.data?.title,
      content: event?.data?.content,
      wiki_type_id: event?.data?.type,
    }
    await useApiCall('/api/wiki', {
      method: 'POST',
      body: requestData,
    })
  } catch (e) {

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