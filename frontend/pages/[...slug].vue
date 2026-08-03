<template>
  <div class="error-page-container">
    <div class="w-full px-4 py-8">
      <UCard class="error-main-card shadow-md">
        <div class="flex flex-col h-full justify-between gap-8 py-4">
          <div>
            <div class="flex items-center justify-center mb-4">
              <UBadge color="error" variant="subtle" size="md" class="uppercase tracking-wider font-semibold justify-center">
                Код 404
              </UBadge>
            </div>
            
            <h1 class="text-3xl font-bold text-[var(--ui-text-highlighted)] mb-4 tracking-tight text-center">
              Страница не найдена
            </h1>
          </div>
          
          <div class="flex flex-wrap gap-3 justify-center">
            <UButton
              to="/"
              color="primary"
              size="sm"
              icon="material-symbols:home-outline"
            >
              На главную
            </UButton>
            <UButton
              color="neutral"
              variant="outline"
              size="sm"
              icon="material-symbols:arrow-back-rounded"
              @click="goBack"
            >
              Назад
            </UButton>
          </div>
        </div>
      </UCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';

// Устанавливаем HTTP-статус 404 для серверного рендеринга (SSR)
if (import.meta.server) {
  setResponseStatus(404);
}

useHead({
  title: '404 — Страница не найдена | Agora RU',
});

const router = useRouter();

const goBack = () => {
  router.back();
};
</script>

<style lang="scss" scoped>
.error-page-container {
  width: 100%;
  min-height: calc(100vh - 10rem);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.error-main-card {
  background-color: var(--ui-bg);
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius, 0) !important;
  
  // Force square corners for Nuxt UI elements inside
  :deep([data-slot="card"]) {
    border-radius: var(--ui-radius, 0) !important;
  }
}
</style>
