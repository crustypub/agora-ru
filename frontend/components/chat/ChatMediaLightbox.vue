<template>
  <UModal v-model:open="isOpen" :title="currentMediaName">
    <template #body>
      <div class="media-lightbox__body">
        <!-- Левая стрелка -->
        <UButton
          v-if="media.length > 1"
          icon="material-symbols:chevron-left"
          variant="ghost"
          color="neutral"
          size="xl"
          class="media-lightbox__nav-btn media-lightbox__nav-btn--prev"
          @click="showPrev"
        />

        <!-- Медиа (картинка или видео) -->
        <div class="media-lightbox__media-wrapper">
          <video
            v-if="isVideo"
            ref="videoPlayerRef"
            :src="currentMediaUrl"
            controls
            autoplay
            playsinline
            class="media-lightbox__video"
          ></video>
          <img
            v-else
            :src="currentMediaUrl"
            class="media-lightbox__img"
            :alt="currentMediaName"
          />
        </div>

        <!-- Правая стрелка -->
        <UButton
          v-if="media.length > 1"
          icon="material-symbols:chevron-right"
          variant="ghost"
          color="neutral"
          size="xl"
          class="media-lightbox__nav-btn media-lightbox__nav-btn--next"
          @click="showNext"
        />
      </div>
    </template>

    <template #footer>
      <div class="media-lightbox__footer">
        <div class="media-lightbox__counter">
          {{ (activeIndex || 0) + 1 }} / {{ media.length }}
        </div>
        <div class="media-lightbox__actions">
          <UButton
            color="neutral"
            variant="subtle"
            icon="material-symbols:download"
            @click="downloadMedia"
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
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';

interface IMediaItem {
  file_url: string;
  file_name: string;
  file_mime: string;
}

const props = defineProps<{
  media: IMediaItem[];
  activeIndex: number | null;
}>();

const emit = defineEmits<{
  (e: 'update:activeIndex', index: number | null): void;
}>();

const videoPlayerRef = ref<HTMLVideoElement | null>(null);

const isOpen = computed({
  get: () => props.activeIndex !== null,
  set: (val) => {
    if (!val) {
      emit('update:activeIndex', null);
    }
  }
});

const currentMedia = computed(() => {
  if (props.activeIndex === null) return null;
  return props.media[props.activeIndex] || null;
});

const currentMediaUrl = computed(() => currentMedia.value?.file_url || '');
const currentMediaName = computed(() => currentMedia.value?.file_name || 'Просмотр');
const currentMediaMime = computed(() => currentMedia.value?.file_mime || '');
const isVideo = computed(() => currentMediaMime.value.startsWith('video/'));

const showPrev = () => {
  if (props.activeIndex === null) return;
  const newIndex = (props.activeIndex - 1 + props.media.length) % props.media.length;
  emit('update:activeIndex', newIndex);
};

const showNext = () => {
  if (props.activeIndex === null) return;
  const newIndex = (props.activeIndex + 1) % props.media.length;
  emit('update:activeIndex', newIndex);
};

const close = () => {
  emit('update:activeIndex', null);
};

// При смене слайда или закрытии — глушим плеер
watch(() => props.activeIndex, () => {
  if (videoPlayerRef.value) {
    videoPlayerRef.value.pause();
  }
});

watch(isOpen, (newVal) => {
  if (!newVal && videoPlayerRef.value) {
    videoPlayerRef.value.pause();
  }
});

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

const downloadMedia = async () => {
  const url = currentMediaUrl.value;
  if (!url) return;
  
  try {
    const response = await fetch(url);
    const blob = await response.blob();
    const blobUrl = window.URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = blobUrl;
    link.download = currentMediaName.value;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    window.URL.revokeObjectURL(blobUrl);
  } catch (error) {
    console.error('Failed to download media:', error);
    window.open(url, '_blank');
  }
};
</script>

<style lang="scss" scoped>
.media-lightbox {
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

  &__media-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
    max-width: 100%;
    max-height: 70vh;
    padding: 0.5rem;
  }

  &__img,
  &__video {
    max-width: 100%;
    max-height: 70vh;
    border-radius: 4px;
  }

  &__img {
    object-fit: contain;
    user-select: none;
  }

  &__video {
    outline: none;
    background-color: #000;
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
