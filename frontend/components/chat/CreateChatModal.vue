<template>
  <UModal v-model:open="isOpen" title="Создать чат">
    <template #body>
      <div class="create-chat-modal__tabs">
        <UButton
          :variant="activeTab === 'direct' ? 'solid' : 'ghost'"
          color="neutral"
          label="Личный чат"
          @click="activeTab = 'direct'"
          class="create-chat-modal__tab-btn"
        />
        <UButton
          :variant="activeTab === 'group' ? 'solid' : 'ghost'"
          color="neutral"
          label="Групповой чат"
          @click="activeTab = 'group'"
          class="create-chat-modal__tab-btn"
        />
      </div>

      <div v-if="activeTab === 'direct'" class="create-chat-modal__form">
        <p class="create-chat-modal__hint">
          Выберите пользователя из списка по имени или username, чтобы начать диалог.
        </p>
        <UFormField label="Пользователь" required>
          <USelectMenu
            v-model="selectedUser"
            v-model:search-term="searchTerm"
            :items="usersItems"
            :loading="searchLoading"
            :ignore-filter="true"
            placeholder="Введите не менее 3-х символов для поиска..."
            option-attribute="label"
            :search-input="{ placeholder: 'Поиск...' }"
            class="w-full"
          >
            <template #empty>
              Пользователи не найдены
            </template>
          </USelectMenu>
        </UFormField>
      </div>

      <div v-else class="create-chat-modal__form">
        <p class="create-chat-modal__hint">
          Создайте новую группу для обсуждения проектов и новостей.
        </p>
        <UFormField label="Название группы" required>
          <UInput
            v-model="groupName"
            placeholder="Например, Разработка Agora-RU"
            autofocus
          />
        </UFormField>
        <UFormField label="Описание группы">
          <UTextarea
            v-model="groupDescription"
            placeholder="О чем этот чат? (необязательно)"
            autoresize
          />
        </UFormField>
      </div>

      <div v-if="errorMessage" class="create-chat-modal__error">
        {{ errorMessage }}
      </div>
    </template>

    <template #footer>
      <div class="create-chat-modal__footer-actions">
        <UButton
          color="neutral"
          variant="soft"
          label="Отмена"
          @click="isOpen = false"
        />
        <UButton
          v-if="activeTab === 'direct'"
          color="primary"
          label="Начать диалог"
          :loading="submitting"
          :disabled="!selectedUser"
          @click="handleCreateDirect"
        />
        <UButton
          v-else
          color="primary"
          label="Создать группу"
          :loading="submitting"
          :disabled="!groupName.trim()"
          @click="handleCreateGroup"
        />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useChat } from '~/composables/useChat';
import { useNotify } from '~/composables/useNotify';

import { useApiCall } from '~/composables/useApi';
import { debounce } from 'lodash-es';

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:open', val: boolean): void;
}>();

const isOpen = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val),
});

const { createDirectChat, createGroupChat } = useChat();
const notify = useNotify();

const activeTab = ref<'direct' | 'group'>('direct');
const submitting = ref(false);
const errorMessage = ref('');

const selectedUser = ref<{ id: string; label: string; avatar?: { src: string; alt?: string } } | null>(null);
const usersItems = ref<{ id: string; label: string; avatar?: { src: string; alt?: string } }[]>([]);
const searchTerm = ref('');
const searchLoading = ref(false);
const groupName = ref('');
const groupDescription = ref('');

const debouncedSearchUsers = debounce(async (query: string) => {
  const trimmed = query.trim();
  if (trimmed.length < 3) {
    usersItems.value = [];
    return;
  }
  searchLoading.value = true;
  try {
    const res = await useApiCall<{ data: any[] }>('/api/users', {
      query: { search_value: trimmed, limit: 5 }
    });
    const rawData = res.data || [];
    const items = rawData.map((u: any) => {
      const name = [u.first_name, u.last_name].filter(Boolean).join(' ') || u.username || 'Без имени';
      const usernameSuffix = u.username ? ` (@${u.username})` : '';
      return {
        id: u.id,
        label: `${name}${usernameSuffix}`,
        avatar: {
          src: u.avatar_url || '',
          alt: name
        }
      };
    });
    // Сохраняем текущего выбранного пользователя в списке
    if (selectedUser.value && !items.some(item => item.id === selectedUser.value?.id)) {
      items.push(selectedUser.value);
    }
    usersItems.value = items;
  } catch (err) {
    console.error('Failed to search users:', err);
    usersItems.value = [];
  } finally {
    searchLoading.value = false;
  }
}, 300);

watch(searchTerm, (newVal) => {
  debouncedSearchUsers(newVal);
});

watch(() => props.open, (newVal) => {
  if (newVal) {
    selectedUser.value = null;
    usersItems.value = [];
    searchTerm.value = '';
  }
});

const handleCreateDirect = async () => {
  if (!selectedUser.value) return;
  submitting.value = true;
  errorMessage.value = '';
  
  const result = await createDirectChat(selectedUser.value.id);
  submitting.value = false;
  
  if (result.success) {
    notify.success('Чат создан', 'Вы можете начать общение.');
    selectedUser.value = null;
    isOpen.value = false;
  } else {
    errorMessage.value = result.error || 'Не удалось создать чат. Проверьте правильность ID/username пользователя.';
  }
};

const handleCreateGroup = async () => {
  if (!groupName.value.trim()) return;
  submitting.value = true;
  errorMessage.value = '';
  
  const result = await createGroupChat(groupName.value.trim(), groupDescription.value.trim());
  submitting.value = false;
  
  if (result.success) {
    notify.success('Групповой чат создан');
    groupName.value = '';
    groupDescription.value = '';
    isOpen.value = false;
  } else {
    errorMessage.value = result.error || 'Не удалось создать группу.';
  }
};
</script>

<style lang="scss" scoped>
.create-chat-modal {
  &__tabs {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
    border-bottom: 1px solid var(--ui-border);
    padding-bottom: 0.75rem;
    margin-bottom: 1rem;
  }

  &__tab-btn {
    justify-content: center;
  }

  &__form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  &__hint {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
  }

  &__error {
    margin-top: 1rem;
    padding: 0.75rem;
    font-size: 0.75rem;
    border-radius: var(--ui-radius, 4px);
    background-color: rgb(254, 242, 242);
    color: rgb(220, 38, 38);
    border: 1px solid rgb(254, 202, 202);
  }

  &__footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    width: 100%;
  }
}
</style>
