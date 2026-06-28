<template>
  <UModal v-model:open="isOpen" title="Создать чат">
    <template #body>
      <!-- Mode Selection Tab Buttons -->
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

      <!-- Direct Chat Creation -->
      <div v-if="activeTab === 'direct'" class="create-chat-modal__form">
        <p class="create-chat-modal__hint">
          Введите UUID или имя пользователя, чтобы начать диалог.
        </p>
        <UFormField label="ID или username пользователя" required>
          <UInput
            v-model="directUserId"
            placeholder="Например, a3b4c5d6..."
            autofocus
            @keyup.enter="handleCreateDirect"
          />
        </UFormField>
      </div>

      <!-- Group Chat Creation -->
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

      <!-- Error alert -->
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
          :disabled="!directUserId.trim()"
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
import { ref, computed } from 'vue';
import { useChat } from '~/composables/useChat';
import { useNotify } from '~/composables/useNotify';

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

// Form fields
const directUserId = ref('');
const groupName = ref('');
const groupDescription = ref('');

const handleCreateDirect = async () => {
  if (!directUserId.value.trim()) return;
  submitting.value = true;
  errorMessage.value = '';
  
  const result = await createDirectChat(directUserId.value.trim());
  submitting.value = false;
  
  if (result.success) {
    notify.success('Чат создан', 'Вы можете начать общение.');
    directUserId.value = '';
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
