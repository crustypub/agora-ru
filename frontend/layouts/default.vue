<template>
  <div class="default-layout">
    <Header />

    <div class="layout-body">
      <!-- Sidebar -->
      <USidebar v-model:open="isSidebarOpen" variant="inset" collapsible="icon" :ui="{
        container: 'top-14 h-[calc(100vh-3.5rem)]'
      }">
        <UNavigationMenu orientation="vertical" :items="navItems" :collapsed="!isSidebarOpen" :tooltip="true" :ui="{
          root: 'w-full group/nav-menu',
          linkLeadingIcon: 'w-5 h-5 min-w-5 shrink-0'
        }" />

        <template #footer>
          <div v-if="authUser" class="w-full">
            <UDropdownMenu :items="sidebarUserItems" :popper="{ placement: 'right-end' }" class="w-full">
              <button
                class="flex items-center gap-2 px-2 py-1.5 w-full overflow-hidden sidebar-footer-inner text-left hover:bg-[var(--ui-bg-hovered)] transition-colors duration-200 cursor-pointer border-0 bg-transparent rounded-none outline-none">
                <UAvatar :src="authUser.avatar_url || ''" :alt="authUser.first_name || authUser.username || ''"
                  size="sm" class="shrink-0" />
                <div class="flex flex-col min-w-0 sidebar-user-info">
                  <span class="text-xs font-semibold truncate text-[var(--ui-text-highlighted)]">
                    {{ authUser.first_name || authUser.username }}
                  </span>
                  <span class="text-[10px] truncate text-[var(--ui-text-muted)]">
                    @{{ authUser.username }}
                  </span>
                </div>
                <UIcon name="material-symbols:more-vert"
                  class="ml-auto shrink-0 sidebar-more-icon text-[var(--ui-text-muted)] w-4 h-4" />
              </button>
            </UDropdownMenu>
          </div>
          <div v-else class="w-full px-2 py-1.5">
            <UButton to="/auth" icon="material-symbols:login-rounded" variant="ghost"
              class="w-full flex items-center gap-2 sidebar-login-btn text-[var(--ui-text-muted)] hover:text-[var(--ui-text-highlighted)] hover:bg-[var(--ui-bg-hovered)]"
              :class="{ 'justify-center px-0': !isSidebarOpen, 'justify-start': isSidebarOpen }">
              <span v-if="isSidebarOpen" class="text-xs font-semibold">Войти</span>
            </UButton>
          </div>
        </template>
      </USidebar>

      <!-- Main Content Area -->
      <main class="layout-main">
        <UContainer class="layout-main__container">
          <slot />
        </UContainer>
      </main>
    </div>

    <MobileNav />
    <UNotifications />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRoute } from '#app';
import { useNavigation } from '~/composables/useNavigation';
import { useAuthUser } from '~/composables/useAuthUser';
import { useSidebar } from '~/composables/useSidebar';

const isSidebarOpen = useSidebar();
const { navItems } = useNavigation();
const authUser = useAuthUser();
const route = useRoute();

// Close sidebar on mobile/tablet viewports upon routing to a new page
watch(() => route.fullPath, () => {
  if (import.meta.client && window.innerWidth < 1024) {
    isSidebarOpen.value = false;
  }
});

const handleLogout = async () => {
  try {
    await useApiCall('/api/auth/logout', { method: 'POST' });
  } catch (e) {
    console.error("Logout request failed:", e);
  }
  const token = useCookie('auth_token');
  token.value = null;
  authUser.value = null;
  navigateTo('/');
};


const handleToSettings = () => {
  navigateTo('/settings');
};


const sidebarUserItems = computed(() => {
  const items = [];

  if (authUser.value) {
    items.push({
      label: `Профиль: ${authUser.value.first_name || authUser.value.username}`,
      icon: 'material-symbols:account-circle-outline',
      disabled: true,
    });
    items.push({
      label: 'Настройки',
      icon: 'material-symbols:settings-outline',
      onSelect: () => handleToSettings(),
    });
    items.push({
      label: 'Выйти',
      icon: 'material-symbols:logout-rounded',
      onSelect: () => handleLogout(),
    });
  }

  return [items];
});
</script>

<style scoped lang="scss">
.default-layout {
  width: 100%;
  height: 100dvh;
  display: flex;
  flex-direction: column;
  font-size: $base-font-size;
  font-family: 'IBM Plex Sans', sans-serif;
  overflow: hidden;

  .layout-body {
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
    width: 100%;
    background-color: var(--ui-bg-muted, #f8f9fa);
    /* Gray background to make inset panels pop */
  }

  .layout-main {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;

    &__container {
      flex: 1 1 auto;
      display: flex;
      flex-direction: column;
      padding-top: .5rem;
      padding-bottom: .5rem;
      width: 100%;
    }

    @media (min-width: 1024px) {
      overflow: hidden;
      height: calc(100% - 2rem);
      margin: 1rem auto;
      background-color: var(--ui-bg);
      border: 1px solid var(--ui-border);
      border-radius: var(--ui-radius, 0);
      box-shadow: var(--ui-shadow-sm, 0 1px 2px 0 rgba(0, 0, 0, 0.05));

      &__container {
        flex: 1;
        overflow-y: auto;
        padding: 1.5rem;
        max-width: 100% !important;
      }
    }
  }

  // Centering avatar and hiding texts/buttons when sidebar is collapsed in icon mode
  :deep([data-state="collapsed"]) {

    .sidebar-user-info,
    .sidebar-more-icon {
      display: none !important;
    }

    .sidebar-footer-inner {
      justify-content: center;
      padding-left: 0;
      padding-right: 0;
      gap: 0;
    }

    // Center navigation menu items when collapsed
    [data-slot="list"] {
      width: 100%;
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    [data-slot="item"] {
      width: 100%;
      display: flex;
      justify-content: center;
    }

    [data-slot="link"] {
      justify-content: center !important;
      width: 2rem !important;
      /* w-8 */
      height: 2rem !important;
      /* h-8 */
      padding: 0 !important;
      margin: 0.25rem auto !important;

      &::before {
        inset: 0 !important;
      }
    }

    [data-slot="linkLeadingIcon"] {
      margin: 0 !important;
    }
  }
}
</style>