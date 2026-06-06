<template>
  <div class="user-form">
    <UCard class="user-settings-card">
      <template #header>
        <div class="user-settings-card__header">
          <h3 class="user-settings-card__title">Личный профиль</h3>
          <p class="user-settings-card__subtitle">
            Управляйте своими личными данными и именем пользователя
          </p>
        </div>
      </template>

      <UForm :schema="schema" :state="state" class="user-settings-form" @submit="handleSubmit">
        <!-- Секция аватара -->
        <div class="avatar-section">
          <UAvatar
            :src="authUser?.avatar_url || ''"
            :alt="authUser?.first_name || authUser?.username || 'User'"
            size="xl"
            class="avatar-section__avatar"
          />
          <div class="avatar-section__content">
            <h4 class="avatar-section__title">Аватар профиля</h4>
            <p class="avatar-section__desc">Поддерживаются форматы JPG, PNG, WebP. Изображение будет оптимизировано.</p>
            <div class="avatar-section__actions">
              <UButton
                size="xs"
                variant="subtle"
                color="neutral"
                :loading="isAvatarLoading"
                @click="triggerFileInput"
              >
                <template #leading>
                  <UIcon name="material-symbols:edit-outline-rounded" />
                </template>
                Загрузить фото
              </UButton>
              <UButton
                v-if="authUser?.avatar_url"
                size="xs"
                variant="subtle"
                color="red"
                :loading="isAvatarLoading"
                @click="deleteAvatar"
              >
                <template #leading>
                  <UIcon name="material-symbols:delete-outline-rounded" />
                </template>
                Удалить
              </UButton>
            </div>
          </div>
          <input
            ref="fileInput"
            type="file"
            accept="image/*"
            style="display: none;"
            @change="handleAvatarUpload"
          />
        </div>

        <div class="user-settings-form__section">
          <UFormField label="Имя пользователя (username)" name="username" class="user-settings-form__field">
            <UInput v-model="state.username" placeholder="Введите username" class="w-full">
              <template #leading>
                <UIcon name="material-symbols:alternate-email-rounded" class="input-icon" />
              </template>
            </UInput>
          </UFormField>
        </div>

        <div class="user-settings-form__row">
          <UFormField label="Имя" name="first_name" class="user-settings-form__col">
            <UInput v-model="state.first_name" placeholder="Введите имя" class="w-full">
              <template #leading>
                <UIcon name="material-symbols:person-outline-rounded" class="input-icon" />
              </template>
            </UInput>
          </UFormField>

          <UFormField label="Фамилия" name="last_name" class="user-settings-form__col">
            <UInput v-model="state.last_name" placeholder="Введите фамилию (необязательно)" class="w-full">
              <template #leading>
                <UIcon name="material-symbols:badge-outline" class="input-icon" />
              </template>
            </UInput>
          </UFormField>
        </div>

        <div class="user-settings-form__actions">
          <Transition name="fade" mode="out-in">
            <span v-if="isSaved" class="user-settings-form__success-message">
              <UIcon name="material-symbols:check-circle-outline-rounded" class="success-icon" />
              Изменения успешно сохранены
            </span>
            <span v-else-if="errorMessage" class="user-settings-form__error-message">
              <UIcon name="material-symbols:error-outline-rounded" class="error-icon" />
              {{ errorMessage }}
            </span>
          </Transition>

          <UButton
            type="submit"
            color="primary"
            :loading="isLoading"
            :disabled="isLoading"
            class="user-settings-form__submit"
          >
            <template #leading>
              <UIcon v-if="!isLoading" name="material-symbols:save-outline" />
            </template>
            Сохранить
          </UButton>
        </div>
      </UForm>
    </UCard>
  </div>
</template>

<script setup lang="ts">
import { z } from 'zod';
import type { IAuthMeValue } from '~/models/api/auth.api';
import type { IUserUpdate } from '~/models/api/user.api';

const { data: response } = await useApi<IAuthMeValue>('/api/auth/me');
const authUser = useAuthUser();

const isLoading = ref(false);
const isSaved = ref(false);
const errorMessage = ref('');

const schema = z.object({
    username: z.string({ message: 'Введите username' })
        .min(1, 'Введите username')
        .regex(/^[a-zA-Z0-9_]+$/, 'Username может содержать только латинские буквы, цифры и нижнее подчеркивание'),
    first_name: z.string({ message: 'Введите имя' })
        .min(1, 'Введите имя'),
    last_name: z.string()
        .max(32, 'Фамилия не должна превышать 32 символа')
        .optional()
});

interface IProps {}

defineProps<IProps>();

const defaultValue = {
    username: response?.value?.data.username || '',
    first_name: response?.value?.data.first_name || '',
    last_name: response?.value?.data.last_name || '',
}

const state = reactive(defaultValue);

const handleSubmit = async function () {
    isLoading.value = true;
    isSaved.value = false;
    errorMessage.value = '';
    try {
        const response = await useApiCall<IUserUpdate>('/api/user', {
            method: 'PATCH',
            body: state,
        });

        if (response.status === 'success') {
            authUser.value = response.data;
            isSaved.value = true;
            setTimeout(() => {
                isSaved.value = false;
            }, 3000);
        } else {
            errorMessage.value = 'Не удалось сохранить изменения. Пожалуйста, попробуйте позже.';
        }
    } catch (e) {
        console.error(e);
        errorMessage.value = 'Произошла ошибка при сохранении изменений. Пожалуйста, попробуйте позже.';
    } finally {
        isLoading.value = false;
    }
}

const isAvatarLoading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

const triggerFileInput = () => {
    fileInput.value?.click();
};

const handleAvatarUpload = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;

    const file = files[0];
    const formData = new FormData();
    formData.append('file', file);

    isAvatarLoading.value = true;
    errorMessage.value = '';

    try {
        const res = await useApiCall<{ status: string, avatar_url: string }>('/api/user/avatar', {
            method: 'POST',
            body: formData,
        });

        if (res.status === 'success') {
            if (authUser.value) {
                authUser.value.avatar_url = res.avatar_url;
            }
            isSaved.value = true;
            setTimeout(() => {
                isSaved.value = false;
            }, 3000);
        } else {
            errorMessage.value = 'Не удалось загрузить аватар.';
        }
    } catch (err) {
        console.error(err);
        errorMessage.value = 'Произошла ошибка при загрузке аватара.';
    } finally {
        isAvatarLoading.value = false;
        if (fileInput.value) {
            fileInput.value.value = '';
        }
    }
};

const deleteAvatar = async () => {
    isAvatarLoading.value = true;
    errorMessage.value = '';
    try {
        const res = await useApiCall<{ status: string }>('/api/user/avatar', {
            method: 'DELETE',
        });

        if (res.status === 'success') {
            if (authUser.value) {
                authUser.value.avatar_url = null;
            }
            isSaved.value = true;
            setTimeout(() => {
                isSaved.value = false;
            }, 3000);
        } else {
            errorMessage.value = 'Не удалось удалить аватар.';
        }
    } catch (err) {
        console.error(err);
        errorMessage.value = 'Произошла ошибка при удалении аватара.';
    } finally {
        isAvatarLoading.value = false;
    }
};
</script>

<style lang="scss" scoped>
.avatar-section {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid var(--ui-border);
  margin-bottom: 0.5rem;
  width: 100%;

  &__avatar {
    width: 4.5rem;
    height: 4.5rem;
    border: 2px solid var(--ui-border);
    background-color: var(--ui-bg-muted);
  }

  &__content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  &__title {
    font-size: $text-sm;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__desc {
    font-size: $text-xs;
    color: var(--ui-text-muted);
  }

  &__actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }
}

.user-form {
  flex: 1;
  display: flex;
  justify-content: flex-start;
  align-items: flex-start;
}

.user-settings-card {
  width: 100%;

  &__header {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  &__title {
    font-size: $text-lg;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__subtitle {
    font-size: $text-xs;
    color: var(--ui-text-muted);
  }
}

.user-settings-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;

  &__section {
    width: 100%;
  }

  &__row {
    display: flex;
    gap: 1rem;
    width: 100%;

    @media (max-width: 640px) {
      flex-direction: column;
      gap: 1.25rem;
    }
  }

  &__col {
    flex: 1;
    min-width: 0;
  }

  &__actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 0.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--ui-border);
  }

  &__submit {
    min-width: 120px;
    font-weight: 500;
  }

  &__success-message {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: $text-sm;
    color: var(--color-success-600, #16a34a);
    font-weight: 500;
  }

  &__error-message {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: $text-sm;
    color: var(--color-error-600, #dc2626);
    font-weight: 500;
  }
}

.input-icon {
  font-size: 1.15rem;
  color: var(--ui-text-muted);
}

.success-icon {
  font-size: 1.1rem;
  color: var(--color-success-600, #16a34a);
}

.error-icon {
  font-size: 1.1rem;
  color: var(--color-error-600, #dc2626);
}

// Анимация плавного появления сообщения об успехе
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
