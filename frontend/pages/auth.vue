<template>
  <div class="page-container items-center justify-center">
    <UCard class="auth-card">
      <template #header>
        <span class="auth-card__title">Авторизация</span>
      </template>

      <div class="flex flex-col gap-6">
        <p class="auth-card__desc">
          Для безопасного входа на сайт Agora RU используйте нашего Telegram-бота. 
          Выберите удобный способ ниже:
        </p>

        <!-- Loading state when fetching token -->
        <div v-if="loading" class="flex flex-col items-center justify-center py-6 gap-3">
          <UIcon name="material-symbols:sync-saved-locally-outline-rounded" class="animate-spin text-4xl text-[var(--ui-primary)]" />
          <span class="text-sm text-[var(--ui-text-muted)]">Подготовка сессии авторизации...</span>
        </div>

        <!-- Error state -->
        <div v-else-if="error" class="flex flex-col items-center gap-4 py-4">
          <div class="w-full p-4 text-sm rounded-lg bg-red-50 dark:bg-red-950 text-red-500 dark:text-red-400 border border-red-200 dark:border-red-900 flex items-start gap-3">
            <UIcon name="material-symbols:error-outline-rounded" class="text-lg shrink-0 mt-0.5" />
            <div>
              <div class="font-semibold mb-1">Ошибка загрузки</div>
              <div>{{ error }}</div>
            </div>
          </div>
          <UButton @click="loadSession" icon="material-symbols:refresh-rounded"> Повторить попытку </UButton>
        </div>

        <!-- Main auth flow -->
        <div v-else class="flex flex-col gap-6">
          <!-- Button Authorization -->
          <div class="flex flex-col gap-2">
            <span class="text-xs font-semibold text-[var(--ui-text-muted)] uppercase tracking-wider">Способ 1: В один клик</span>
            <UButton
              :to="botLink"
              target="_blank"
              icon="material-symbols:send-rounded"
              size="lg"
              block
              class="tg-login-btn font-semibold text-white bg-[#54a9eb] hover:bg-[#4397d9] transition-all duration-300"
            >
              Войти через Telegram
            </UButton>
          </div>

          <!-- Divider -->
          <div class="relative flex items-center justify-center">
            <div class="absolute inset-0 flex items-center"><span class="w-full border-t border-[var(--ui-border)]"></span></div>
            <span class="relative px-3 text-xs bg-[var(--ui-bg)] text-[var(--ui-text-muted)] font-medium">ИЛИ</span>
          </div>

          <!-- Manual Code Authorization -->
          <div class="flex flex-col gap-2">
            <span class="text-xs font-semibold text-[var(--ui-text-muted)] uppercase tracking-wider">Способ 2: Вручную через команду</span>
            <p class="text-xs text-[var(--ui-text-muted)]">
              Откройте Telegram, найдите бота <span class="font-semibold">@{{ botUsername }}</span> и отправьте ему следующую команду:
            </p>
            <div class="flex items-center gap-2 p-2 bg-[var(--ui-bg-muted)] border border-[var(--ui-border)] rounded-md font-mono text-sm overflow-x-auto relative">
              <span class="select-all text-[var(--ui-text-highlighted)]">{{ manualCommand }}</span>
              <UButton
                @click="copyCommand"
                variant="ghost"
                color="neutral"
                size="sm"
                class="ml-auto shrink-0"
                :icon="copied ? 'material-symbols:check-circle-outline-rounded' : 'material-symbols:content-copy-outline-rounded'"
              >
                {{ copied ? 'Скопировано' : 'Копировать' }}
              </UButton>
            </div>
          </div>

          <!-- Status / Progress Indicator -->
          <div class="mt-4 p-4 border border-[var(--ui-border)] bg-[var(--ui-bg-muted)] rounded-md flex flex-col gap-3">
            <div class="flex items-center gap-2">
              <UIcon name="material-symbols:hourglass-top-rounded" class="animate-pulse text-[var(--ui-primary)]" />
              <span class="text-sm font-semibold text-[var(--ui-text-highlighted)]">
                Ожидаем подтверждения в Telegram...
              </span>
            </div>
            <UProgress animation="carousel" color="primary" size="xs" />
          </div>
        </div>
      </div>
    </UCard>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useApiCall } from '~/composables/useApi';
import { useCheckAuth } from '~/composables/useCheckAuth';

const token = ref<string>('');
const botUsername = ref<string>('');
const loading = ref<boolean>(true);
const error = ref<string>('');
const copied = ref<boolean>(false);
let checkInterval: any = null;

const { checkAuth } = useCheckAuth();

const botLink = computed(() => {
  return `https://t.me/${botUsername.value}?start=${token.value}`;
});

const manualCommand = computed(() => {
  return `/start ${token.value}`;
});

const loadSession = async () => {
  loading.value = true;
  error.value = '';
  try {
    const data = await useApiCall<{ token: string; bot_username: string }>('/api/auth/telegram/request');
    token.value = data.token;
    botUsername.value = data.bot_username;
    loading.value = false;
    
    // Start polling status
    startPolling();
  } catch (e: any) {
    console.error(e);
    error.value = 'Не удалось связаться с сервером. Пожалуйста, убедитесь, что бэкэнд запущен.';
    loading.value = false;
  }
};

const copyCommand = async () => {
  try {
    await navigator.clipboard.writeText(manualCommand.value);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy text: ', err);
  }
};

const startPolling = () => {
  if (checkInterval) clearInterval(checkInterval);
  checkInterval = setInterval(async () => {
    try {
      const response = await useApiCall<{ status: string }>('/api/auth/telegram/check', {
        method: 'POST',
        body: { token: token.value }
      });
      
      if (response.status === 'success') {
        clearInterval(checkInterval);
        // Successful login: refresh auth user details and redirect
        await checkAuth(true);
        await navigateTo('/', { replace: true });
      } else if (response.status === 'expired') {
        clearInterval(checkInterval);
        // Refresh token if expired
        loadSession();
      }
    } catch (e) {
      console.error('Error polling status: ', e);
    }
  }, 2000);
};

onMounted(async () => {
  const user = await checkAuth();
  if (user) {
    await navigateTo('/', { replace: true });
    return;
  }
  loadSession();
});

onUnmounted(() => {
  if (checkInterval) clearInterval(checkInterval);
});
</script>


<style lang="scss" scoped>
.auth-card {
  width: 100%;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);

  &__title {
    font-size: $text-xl;
    font-weight: 600;
    color: var(--ui-text-highlighted);
  }

  &__desc {
    font-size: $text-sm;
    color: var(--ui-text-muted);
    line-height: 1.5;
  }
}
</style>
