<template>
  <UModal v-model:open="isOpen" :title="currentImageName">
    <template #body>
      <div class="image-lightbox__body">
        <UButton
          v-if="images.length > 1"
          icon="material-symbols:chevron-left"
          variant="ghost"
          color="neutral"
          size="xl"
          class="image-lightbox__nav-btn image-lightbox__nav-btn--prev"
          @click="showPrev"
        />

        <div class="image-lightbox__image-wrapper">
          <img
            :src="currentImageUrl"
            class="image-lightbox__img"
            :alt="currentImageName"
          />
        </div>

        <!-- Правая стрелка -->
        <UButton
          v-if="images.length > 1"
          icon="material-symbols:chevron-right"
          variant="ghost"
          color="neutral"
          size="xl"
          class="image-lightbox__nav-btn image-lightbox__nav-btn--next"
          @click="showNext"
        />
      </div>
    </template>

    <template #footer>
      <div class="image-lightbox__footer">
        <div class="image-lightbox__counter">
          {{ (activeIndex || 0) + 1 }} / {{ images.length }}
        </div>
        <div class="image-lightbox__actions">
          <UButton
            color="neutral"
            variant="subtle"
            icon="material-symbols:download"
            @click="downloadImage"
          >
            Скачать
          </UButton>
          <UButton color="neutral" variant="ghost" @click="close">
            Закрыть
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

interface IImage {
  file_url: string;
  file_name: string;
}

const props = defineProps<{
  images: IImage[];
  activeIndex: number | null;
}>();

const emit = defineEmits<{
  (e: 'update:activeIndex', index: number | null): void;
}>();

const isOpen = computed({
  get: () => props.activeIndex !== null,
  set: (val) => {
    if (!val) {
      emit('update:activeIndex', null);
    }
  }
});

const currentImageUrl = computed(() => {
  if (props.activeIndex === null) return '';
  const img = props.images[props.activeIndex];
  return img ? img.file_url : '';
});

const currentImageName = computed(() => {
  if (props.activeIndex === null) return 'Просмотр';
  const img = props.images[props.activeIndex];
  return img ? img.file_name : 'Просмотр';
});

const showPrev = () => {
  if (props.activeIndex === null) return;
  const newIndex = (props.activeIndex - 1 + props.images.length) % props.images.length;
  emit('update:activeIndex', newIndex);
};

const showNext = () => {
  if (props.activeIndex === null) return;
  const newIndex = (props.activeIndex + 1) % props.images.length;
  emit('update:activeIndex', newIndex);
};

const close = () => {
  emit('update:activeIndex', null);
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (props.activeIndex === null) return;
  if (e.key === 'ArrowLeft') {
    showPrev();
  } else if (e.key === 'ArrowRight') {
    showNext();
  } else if (e.key === 'Escape') {
    close();
  }
};

onMounted(() => {
  if (import.meta.client) {
    window.addEventListener('keydown', handleKeyDown);
  }
});

onUnmounted(() => {
  if (import.meta.client) {
    window.removeEventListener('keydown', handleKeyDown);
  }
});

const downloadImage = async () => {
  const url = currentImageUrl.value;
  if (!url) return;
  
  try {
    const response = await fetch(url);
    const blob = await response.blob();
    const blobUrl = window.URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = blobUrl;
    link.download = currentImageName.value;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    window.URL.revokeObjectURL(blobUrl);
  } catch (error) {
    console.error('Failed to download image:', error);
    window.open(url, '_blank');
  }
};
</script>

<style lang="scss" scoped>
.image-lightbox {
  &__body {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    min-height: 200px;
    background-color: var(--ui-bg-elevated);
    border-radius: 6px;
    overflow: hidden;
  }

  &__image-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
    max-width: 100%;
    max-height: 70vh;
    padding: 0.5rem;
  }

  &__img {
    max-width: 100%;
    max-height: 70vh;
    object-fit: contain;
    border-radius: 4px;
    user-select: none;
  }

  &__nav-btn {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    z-index: 10;
    background-color: rgba(var(--ui-bg), 0.5) !important;
    backdrop-filter: blur(4px);
    border-radius: 50%;
    padding: 0.5rem;
    height: auto;
    width: auto;

    &:hover {
      background-color: rgba(var(--ui-bg), 0.8) !important;
    }

    &--prev {
      left: 0.5rem;
    }

    &--next {
      right: 0.5rem;
    }
  }

  &__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }

  &__counter {
    font-size: 0.875rem;
    color: var(--ui-text-muted);
    font-weight: 500;
  }

  &__actions {
    display: flex;
    gap: 0.5rem;
  }
}
</style>
