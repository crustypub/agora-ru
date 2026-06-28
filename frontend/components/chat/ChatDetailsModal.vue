<template>
  <UModal v-model:open="isOpen" :title="isDirect ? 'Информация о пользователе' : 'Управление группой'">
    <template #body>
      <div v-if="isDirect && directUser" class="chat-details__direct">
        <UAvatar
          :src="directUser.avatar_url || ''"
          :alt="directUserName"
          size="xl"
          class="chat-details__avatar chat-details__avatar--direct"
        >
          <template v-if="!directUser.avatar_url">
            {{ directUserName.charAt(0).toUpperCase() }}
          </template>
        </UAvatar>
        
        <div class="chat-details__info">
          <h3 class="chat-details__title">{{ directUserName }}</h3>
          <span class="chat-details__username">@{{ directUser.username }}</span>
        </div>

        <div class="chat-details__fields">
          <div class="chat-details__field-row">
            <span class="chat-details__field-label">ID пользователя:</span>
            <span class="chat-details__field-value">{{ directUser.id }}</span>
          </div>
        </div>
      </div>

      <div v-else-if="room" class="chat-details__group">
        <div class="chat-details__group-info">
          <UAvatar
            :alt="room.name || 'G'"
            size="lg"
            class="chat-details__avatar chat-details__avatar--group"
          >
            {{ (room.name || 'G').charAt(0).toUpperCase() }}
          </UAvatar>
          <div class="chat-details__group-text">
            <h3 class="chat-details__group-title">{{ room.name }}</h3>
            <p class="chat-details__group-desc" v-if="room.description">
              {{ room.description }}
            </p>
          </div>
        </div>

        <div v-if="isOwnerOrModerator" class="chat-details__section">
          <span class="chat-details__section-title">Добавить участника</span>
          <div class="chat-details__input-group">
            <UInput
              v-model="newMemberId"
              placeholder="UUID или username пользователя"
              class="chat-details__input-field"
              size="sm"
              @keyup.enter="handleAddMember"
            />
            <UButton
              label="Добавить"
              color="primary"
              size="sm"
              :loading="addingMember"
              :disabled="!newMemberId.trim()"
              @click="handleAddMember"
            />
          </div>
        </div>

        <div class="chat-details__section">
          <div class="chat-details__section-header">
            <span class="chat-details__section-title">
              Участники ({{ members.length }})
            </span>
          </div>

          <div class="chat-details__members">
            <div
              v-for="member in members"
              :key="member.id"
              class="chat-details__member"
            >
              <div class="chat-details__member-info">
                <UAvatar
                  :src="member.avatar_url || ''"
                  :alt="member.username"
                  size="xs"
                  class="chat-details__avatar chat-details__avatar--member"
                >
                  <template v-if="!member.avatar_url">
                    {{ member.username.charAt(0).toUpperCase() }}
                  </template>
                </UAvatar>
                <div class="chat-details__member-text">
                  <span class="chat-details__member-name">
                    {{ formatUserName(member) }}
                  </span>
                  <span class="chat-details__member-username">@{{ member.username }}</span>
                </div>
              </div>

              <div class="chat-details__member-actions">
                <UBadge :color="getRoleColor(member.role)" variant="subtle" size="xs">
                  {{ getRoleName(member.role) }}
                </UBadge>

                <UButton
                  v-if="canIKick(member)"
                  icon="material-symbols:close-rounded"
                  color="red"
                  variant="ghost"
                  size="xs"
                  class="chat-details__kick-btn"
                  title="Исключить участника"
                  @click="handleKickMember(member)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template #footer>
      <div class="chat-details__footer">
        <!-- Leave room button (only for groups) -->
        <UButton
          v-if="!isDirect && room"
          label="Выйти из группы"
          color="red"
          variant="soft"
          size="sm"
          :loading="leaving"
          @click="handleLeaveGroup"
        />
        <div v-else></div>

        <UButton
          color="neutral"
          variant="solid"
          label="Закрыть"
          size="sm"
          @click="isOpen = false"
        />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useChat } from '~/composables/useChat';
import { useAuthUser } from '~/composables/useAuthUser';
import { useNotify } from '~/composables/useNotify';
import type { IRoomMemberInfo, IChatListItem } from '~/models/entities/chat.entities';
import { formatUserName } from '~/helpers/chat';

const props = defineProps<{
  open: boolean;
  room: IChatListItem | null;
}>();

const emit = defineEmits<{
  (e: 'update:open', val: boolean): void;
}>();

const isOpen = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val),
});

const authUser = useAuthUser();
const { members, inviteMember, kickMember, leaveRoom } = useChat();
const notify = useNotify();

const addingMember = ref(false);
const newMemberId = ref('');
const leaving = ref(false);

// Сбрасываем поле ввода при закрытии модалки или смене комнаты
watch([() => props.open, () => props.room?.id], () => {
  newMemberId.value = '';
});

const isDirect = computed(() => props.room?.room_type === 'direct');

// Direct chat details helpers
const directUser = computed(() => props.room?.direct_user || null);
const directUserName = computed(() => {
  return directUser.value ? formatUserName(directUser.value) : 'Пользователь';
});

// Roles check for groups
const myRole = computed(() => {
  if (!authUser.value || isDirect.value) return 'member';
  const me = members.value.find(m => m.id === authUser.value?.id);
  return me?.role || 'member';
});

const isOwnerOrModerator = computed(() => {
  return myRole.value === 'owner' || myRole.value === 'moderator';
});

const canIKick = (member: IRoomMemberInfo) => {
  if (isDirect.value) return false;
  if (!authUser.value || member.id === authUser.value.id) return false; 
  if (member.role === 'owner') return false; 
  
  if (myRole.value === 'owner') return true;
  if (myRole.value === 'moderator' && member.role === 'member') return true;
  return false;
};

const getRoleName = (role: string) => {
  switch (role) {
    case 'owner': return 'Владелец';
    case 'moderator': return 'Модератор';
    default: return 'Участник';
  }
};

const getRoleColor = (role: string) => {
  switch (role) {
    case 'owner': return 'red';
    case 'moderator': return 'amber';
    default: return 'neutral';
  }
};

const handleAddMember = async () => {
  if (!props.room || !newMemberId.value.trim()) return;
  addingMember.value = true;
  
  const result = await inviteMember(props.room.id, newMemberId.value.trim());
  addingMember.value = false;
  
  if (result.success) {
    notify.success('Участник добавлен');
    newMemberId.value = '';
  } else {
    notify.error('Ошибка добавления', result.error || 'Проверьте правильность UUID/username.');
  }
};

const handleKickMember = async (member: IRoomMemberInfo) => {
  if (!props.room) return;
  
  const result = await kickMember(props.room.id, member.id);
  if (result.success) {
    notify.success('Участник исключен');
  } else {
    notify.error('Ошибка', result.error || 'Не удалось исключить участника.');
  }
};

const handleLeaveGroup = async () => {
  if (!props.room) return;
  
  leaving.value = true;
  const result = await leaveRoom(props.room.id);
  leaving.value = false;
  
  if (result.success) {
    notify.success('Вы вышли из группы');
    isOpen.value = false;
  } else {
    notify.error('Ошибка', result.error || 'Не удалось выйти из группы.');
  }
};
</script>

<style lang="scss" scoped>
.chat-details {
  &__direct {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 1rem 0;
  }

  &__avatar {
    background-color: var(--ui-primary-subtle);
    color: var(--ui-primary);
    
    &--direct {
      border-radius: 9999px;
      font-size: 1.25rem;
    }

    &--group {
      border-radius: var(--ui-radius, 4px);
      font-size: 1.125rem;
    }

    &--member {
      border-radius: 9999px;
    }
  }

  &__info {
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  &__title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__username {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    font-family: monospace;
    user-select: all;
  }

  &__fields {
    width: 100%;
    border-top: 1px solid var(--ui-border);
    padding-top: 1rem;
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  &__field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
  }

  &__field-label {
    color: var(--ui-text-muted);
  }

  &__field-value {
    font-family: monospace;
    user-select: all;
    color: var(--ui-text-highlighted);
  }

  &__group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  &__group-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  &__group-text {
    flex: 1;
    min-width: 0;
  }

  &__group-title {
    font-weight: 600;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--ui-text-highlighted);
  }

  &__group-desc {
    font-size: 0.75rem;
    color: var(--ui-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__section {
    border-top: 1px solid var(--ui-border);
    padding-top: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  &__section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  &__section-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__input-group {
    display: flex;
    gap: 0.5rem;
  }

  &__input-field {
    flex: 1;
  }

  &__members {
    max-height: 220px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-right: 0.25rem;
  }

  &__member {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem;
    border-radius: var(--ui-radius, 4px);
    border: 1px solid rgba(var(--ui-border), 0.5);
    background-color: rgba(var(--ui-bg-muted), 0.2);
  }

  &__member-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  &__member-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  &__member-name {
    font-size: 0.75rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--ui-text-highlighted);
  }

  &__member-username {
    font-size: 0.625rem;
    color: var(--ui-text-muted);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__member-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  &__kick-btn {
    &:hover {
      background-color: rgb(254, 242, 242);
    }
  }

  &__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }
}
</style>
