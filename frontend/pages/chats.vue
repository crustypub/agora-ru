<template>
  <div class="chat-page-wrapper">
    <!-- Skeleton: WS ещё не подключён и комнаты ещё не загружены -->
    <div v-if="showSkeleton" class="chat-skeleton-wrapper">
      <!-- Skeleton header -->
      <div class="chat-skeleton__header">
        <USkeleton class="chat-skeleton__title-bar" />
      </div>

      <!-- Skeleton body (sidebar + area) -->
      <div class="chat-skeleton__body">
        <!-- Sidebar skeleton -->
        <div class="chat-skeleton__sidebar">
          <USkeleton class="chat-skeleton__search-bar" />
          <div v-for="i in 6" :key="i" class="chat-skeleton__item">
            <USkeleton class="chat-skeleton__avatar" />
            <div class="chat-skeleton__lines">
              <USkeleton class="chat-skeleton__line chat-skeleton__line--name" />
              <USkeleton class="chat-skeleton__line chat-skeleton__line--msg" />
            </div>
          </div>
        </div>

        <!-- Chat area skeleton (desktop only) -->
        <div class="chat-skeleton__area">
          <div class="chat-skeleton__area-content">
            <USkeleton class="chat-skeleton__placeholder-avatar" />
            <USkeleton class="chat-skeleton__line chat-skeleton__line--placeholder-1" />
            <USkeleton class="chat-skeleton__line chat-skeleton__line--placeholder-2" />
          </div>
        </div>
      </div>
    </div>

    <!-- Нормальный вид после подключения -->
    <template v-else>
      <!-- Chats layout container -->
      <div class="chat-layout">
        <!-- Sidebar: Shown on desktop OR on mobile when no room is selected -->
        <ChatSidebar
          v-if="isLargeScreen || !activeRoomId"
          class="chat-layout__sidebar"
        />

        <!-- Message Area: Shown on desktop OR on mobile when a room is selected -->
        <ChatArea
          v-if="isLargeScreen || activeRoomId"
          class="chat-layout__area"
          @back="selectRoom(null)"
          :is-mobile="!isLargeScreen"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useAuthUser } from '~/composables/useAuthUser';
import { useChat } from '~/composables/useChat';
import ChatSidebar from '~/components/chat/ChatSidebar.vue';
import ChatArea from '~/components/chat/ChatArea.vue';

useHead({
  title: 'Agora RU — Чаты',
});

const authUser = useAuthUser();
const { isWsConnected, activeRoomId, selectRoom, rooms, roomsLoading } = useChat();

// Skeleton показывается пока WS не подключён И список комнат пуст
// (rooms.length > 0 означает что данные уже есть из прошлой навигации)
const showSkeleton = computed(() => !isWsConnected.value && rooms.value.length === 0);

const isLargeScreen = ref(true);

const checkScreenSize = () => {
  if (import.meta.client) {
    isLargeScreen.value = window.innerWidth >= 1024;
  }
};

onMounted(async () => {
  // Guard: redirect if not authenticated
  if (!authUser.value) {
    await navigateTo('/auth');
    return;
  }

  checkScreenSize();
  window.addEventListener('resize', checkScreenSize);
  // WS-соединение и fetchRooms теперь управляются глобально через plugins/chat.client.ts
});

onUnmounted(() => {
  if (import.meta.client) {
    window.removeEventListener('resize', checkScreenSize);
  }
  activeRoomId.value = null;
  // Не закрываем WS здесь — соединение живёт на уровне приложения
});
</script>

<style lang="scss">
/* Подстройка глобального layout под полноэкранный чат */
.layout-main:has(.chat-page-wrapper) {
  overflow: hidden !important;

  @media (max-width: 1023px) {
    height: 100% !important;
  }
}

.layout-main__container:has(.chat-page-wrapper) {
  padding: 0 !important;
  max-width: 100% !important;
  height: 100% !important;
  overflow: hidden !important;
  flex: 1 1 100% !important;
}
</style>

<style lang="scss" scoped>
.chat-page-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius, 8px);
  background-color: var(--ui-bg);
}

.chat-skeleton-wrapper {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
}

.chat-skeleton {
  &__header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--ui-border);
  }

  &__title-bar {
    height: 1.25rem;
    width: 5rem;
    border-radius: var(--ui-radius, 4px);
  }

  &__body {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  &__sidebar {
    width: 100%;
    flex-shrink: 0;
    border-right: 1px solid var(--ui-border);
    display: flex;
    flex-direction: column;
    padding: 0.75rem;

    @media (min-width: 1024px) {
      width: 350px;
    }
  }

  &__search-bar {
    height: 2rem;
    width: 100%;
    border-radius: var(--ui-radius, 4px);
    margin-bottom: 0.75rem;
  }

  &__item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem;
  }

  &__avatar {
    height: 2.5rem;
    width: 2.5rem;
    border-radius: 9999px;
    flex-shrink: 0;
  }

  &__lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  &__line {
    border-radius: var(--ui-radius, 4px);

    &--name {
      height: 0.875rem;
      width: 75%;
    }

    &--msg {
      height: 0.75rem;
      width: 50%;
    }

    &--placeholder-1 {
      height: 1rem;
      width: 10rem;
    }

    &--placeholder-2 {
      height: 0.75rem;
      width: 16rem;
    }
  }

  &__area {
    display: none;
    flex-direction: column;
    flex: 1;
    gap: 1rem;
    padding: 1.5rem;

    @media (min-width: 1024px) {
      display: flex;
    }
  }

  &__area-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.75rem;
    text-align: center;
  }

  &__placeholder-avatar {
    height: 3.5rem;
    width: 3.5rem;
    border-radius: 9999px;
  }
}

.chat-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  width: 100%;
  height: 100%;
  background-color: var(--ui-bg);
  overflow: hidden;

  &__sidebar {
    width: 100%;
    flex-shrink: 0;
    border-right: 1px solid var(--ui-border);
    height: 100%;

    @media (min-width: 1024px) {
      width: 350px;
    }
  }

  &__area {
    flex: 1;
    height: 100%;
    min-width: 0;
  }
}
</style>
