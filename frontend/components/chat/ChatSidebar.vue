<template>
  <div class="chat-sidebar">
    <!-- Top toolbar: Search & Create Button -->
    <div class="chat-sidebar__toolbar">
      <UInput
        v-model="searchQuery"
        placeholder="Поиск чатов..."
        icon="material-symbols:search-rounded"
        class="chat-sidebar__search-input"
        size="sm"
        clearable
      />
      <UButton
        icon="material-symbols:add-rounded"
        color="primary"
        size="sm"
        @click="isCreateModalOpen = true"
        title="Создать новый чат"
      />
    </div>

    <!-- Rooms list -->
    <div
      ref="scrollContainer"
      class="chat-sidebar__rooms"
      @scroll="handleScroll"
    >
      <div v-if="roomsLoading && rooms.length === 0" class="chat-sidebar__loading">
        <UIcon name="material-symbols:sync-saved-locally-outline-rounded" class="chat-sidebar__spinner" />
        <span class="chat-sidebar__loading-text">Загрузка комнат...</span>
      </div>

      <template v-else-if="rooms.length > 0">
        <ChatRoomItem
          v-for="room in rooms"
          :key="room.id"
          :room="room"
          :is-active="activeRoomId === room.id"
          @select="selectRoom(room.id)"
        />
        
        <!-- Loading indicator for paginated pages -->
        <div v-if="roomsLoading" class="chat-sidebar__pagination-loading">
          <UIcon name="material-symbols:sync-saved-locally-outline-rounded" class="chat-sidebar__spinner" />
        </div>
      </template>

      <div v-else class="chat-sidebar__empty">
        <UIcon name="material-symbols:chat-bubble-outline" class="chat-sidebar__empty-icon" />
        <div class="chat-sidebar__empty-title">Чатов не найдено</div>
        <p class="chat-sidebar__empty-desc">
          {{ searchQuery ? 'Попробуйте изменить запрос' : 'Создайте первый личный или групповой чат' }}
        </p>
        <UButton
          v-if="!searchQuery"
          label="Создать чат"
          size="sm"
          color="primary"
          variant="soft"
          @click="isCreateModalOpen = true"
        />
      </div>
    </div>

    <!-- Create chat Modal -->
    <CreateChatModal v-model:open="isCreateModalOpen" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useChat } from '~/composables/useChat';
import ChatRoomItem from './ChatRoomItem.vue';
import CreateChatModal from './CreateChatModal.vue';

const { 
  rooms, 
  activeRoomId, 
  roomsLoading, 
  roomsMeta, 
  fetchRooms, 
  selectRoom 
} = useChat();

const searchQuery = ref('');
const isCreateModalOpen = ref(false);
const scrollContainer = ref<HTMLElement | null>(null);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Watch search query changes to trigger API load with debounce
watch(searchQuery, (newVal) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    fetchRooms(1, false, newVal);
  }, 350);
});

// Scroll pagination handler
const handleScroll = async (e: Event) => {
  const target = e.target as HTMLElement;
  
  // Check if scrolled near the bottom (within 20px)
  const isNearBottom = target.scrollHeight - target.scrollTop - target.clientHeight < 20;
  
  if (isNearBottom && !roomsLoading.value && roomsMeta.value?.has_next) {
    const nextPage = roomsMeta.value.current_page + 1;
    await fetchRooms(nextPage, true, searchQuery.value);
  }
};
</script>

<style lang="scss" scoped>
.chat-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--ui-bg);

  &__toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--ui-border);
  }

  &__search-input {
    flex: 1;
  }

  &__rooms {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    -webkit-overflow-scrolling: touch;
  }

  &__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    gap: 0.5rem;
  }

  &__spinner {
    animation: spin 1s linear infinite;
    font-size: 1.5rem;
    color: var(--ui-text-muted);
  }

  &__loading-text {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
  }

  &__pagination-loading {
    padding: 1rem;
    text-align: center;
    
    .chat-sidebar__spinner {
      font-size: 1.125rem;
    }
  }

  &__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
    gap: 0.75rem;
  }

  &__empty-icon {
    font-size: 2.5rem;
    color: var(--ui-text-muted);
  }

  &__empty-title {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--ui-text-highlighted);
  }

  &__empty-desc {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    max-width: 200px;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
