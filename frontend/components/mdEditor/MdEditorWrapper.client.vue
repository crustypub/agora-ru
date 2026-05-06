<template>
  <form class="md-editor-form" @submit.prevent="handleSubmit">

    <USelect
      v-model="selectedType"
      :items="typeOptions"
      value-attribute="value"
      option-attribute="label"
      placeholder="Выберите тип статьи"
      :required="true"
      class="md-editor-form__select"
    />

    <ClientOnly>
      <MdEditor
        v-model="markdownContent"
        language="en-US"
        :toolbars="activeToolbars"
        class="md-editor__wrapper"
      />
    </ClientOnly>

    <UButton
      type="submit"
      :disabled="!selectedType || !markdownContent.trim()"
      class="md-editor-form__submit"
    >
      Сохранить
    </UButton>

  </form>
</template>

<script setup lang="ts">
import { MdEditor } from 'md-editor-v3';
import type { ToolbarNames } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import type { IWikiTypeResponseItem } from '~/models/entities/wiki.entities';

interface Props {
  wikiTypes: IWikiTypeResponseItem[] | undefined;
}

const props = defineProps<Props>();

const markdownContent = ref('');
const selectedType = ref<string | undefined>(undefined);

const typeOptions = computed(() =>
  (props.wikiTypes ?? []).map((t) => ({ value: t.id, label: t.title }))
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

function handleSubmit() {
  // TODO: заполни обработчик самостоятельно
}
</script>

<style lang="scss" scoped>
.md-editor-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;

  &__select {
    width: 100%;
  }

  &__submit {
    align-self: flex-end;
  }
}

.md-editor__wrapper {
  flex: 1;

  // Увеличиваем зону касания кнопок тулбара на мобильных
  @media (max-width: 768px) {
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