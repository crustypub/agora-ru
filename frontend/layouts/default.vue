<template>
  <div class="default-layout">
    <Header />

    <div class="layout-body">
      <!-- Sidebar -->
      <USidebar
        v-model:open="isSidebarOpen"
        variant="inset"
        collapsible="icon"
        :ui="{
          container: 'top-14 h-[calc(100vh-3.5rem)]'
        }"
      >
        <!-- Sidebar links -->
        <UNavigationMenu
          orientation="vertical"
          :items="navItems"
          class="px-2"
          :collapsed="!isSidebarOpen"
          :tooltip="true"
          :ui="{
            root: 'w-full group/nav-menu',
            link: 'font-medium text-sm group-data-[collapsed=true]/nav-menu:justify-center group-data-[collapsed=true]/nav-menu:w-8 group-data-[collapsed=true]/nav-menu:h-8 group-data-[collapsed=true]/nav-menu:p-0 group-data-[collapsed=true]/nav-menu:mx-auto group-data-[collapsed=true]/nav-menu:before:inset-0',
            linkLeadingIcon: 'w-5 h-5 min-w-5 shrink-0'
          }"
        />

        <template #footer v-if="authUser">
          <div class="flex items-center gap-2 px-2 py-1.5 w-full overflow-hidden sidebar-footer-inner">
            <UAvatar
              :src="authUser.avatar_url || ''"
              :alt="authUser.first_name || authUser.username || ''"
              size="sm"
              class="shrink-0"
            />
            <div class="flex flex-col min-w-0 sidebar-user-info">
              <span class="text-xs font-semibold truncate text-[var(--ui-text-highlighted)]">
                {{ authUser.first_name || authUser.username }}
              </span>
              <span class="text-[10px] truncate text-[var(--ui-text-muted)]">
                @{{ authUser.username }}
              </span>
            </div>
            <UButton
              icon="material-symbols:logout-rounded"
              variant="ghost"
              color="red"
              size="xs"
              class="ml-auto shrink-0 sidebar-logout-btn"
              @click="handleLogout"
            />
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
  </div>
</template>

<script setup lang="ts">
import { useNavigation } from '~/composables/useNavigation';
import { useAuthUser } from '~/composables/useAuthUser';

const isSidebarOpen = useState('isSidebarOpen', () => true);
const { navItems } = useNavigation();
const authUser = useAuthUser();

const handleLogout = () => {
    const token = useCookie('auth_token');
    token.value = null;
    authUser.value = null;
    navigateTo('/');
};
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
    background-color: var(--ui-bg-muted, #f8f9fa); /* Gray background to make inset panels pop */
  }

  .layout-main {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;

    &__container {
      flex: 1 1 auto;
      display: flex;
      flex-direction: column;
      padding-top: .5rem;
      padding-bottom: .5rem;
    }

    @media (min-width: 1024px) {
      overflow: hidden;
      height: calc(100% - 2rem);
      margin: 1rem 1rem 1rem 0;
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
    .sidebar-logout-btn {
      display: none !important;
    }

    .sidebar-footer-inner {
      justify-content: center;
      padding-left: 0;
      padding-right: 0;
      gap: 0;
    }
  }
}
</style>