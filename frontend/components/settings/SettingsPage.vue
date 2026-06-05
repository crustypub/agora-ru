<template>
  <div class="settings">
    <!-- Desktop Layout (>= 768px) -->
    <template v-if="!isMobile">
      <div class="settings__tabs">
        <UTabs v-model="active" orientation="vertical" size="md" variant="link" :items="items" class="w-full" />
      </div>
      <div class="settings__content">
        <UserSettingTab v-if="active === '0'"/>
      </div>
    </template>

    <!-- Mobile Layout (< 768px) -->
    <template v-else>
      <Transition name="fade-slide" mode="out-in">
        <!-- Mobile List View -->
        <div v-if="!showMobileDetail" key="list" class="settings__mobile-list">
          <div class="settings__mobile-header">
            <h2 class="settings__mobile-title">Настройки</h2>
          </div>
          <div class="settings__mobile-items">
            <button
              v-for="(item, index) in items"
              :key="index"
              class="settings__mobile-item"
              @click="selectTab(String(index))"
            >
              <div class="settings__mobile-item-left">
                <div class="settings__mobile-item-icon-wrapper">
                  <UIcon 
                    :name="item.icon || 'material-symbols:settings-outline'" 
                    class="settings__mobile-item-icon" 
                    :style="item.iconColor ? { color: item.iconColor } : {}"
                  />
                </div>
                <span class="settings__mobile-item-label">{{ item.label }}</span>
              </div>
              <UIcon name="material-symbols:chevron-right-rounded" class="settings__mobile-item-arrow" />
            </button>
          </div>
        </div>

        <!-- Mobile Detail View -->
        <div v-else key="detail" class="settings__mobile-detail">
          <div class="settings__mobile-detail-header">
            <UButton
              variant="ghost"
              color="neutral"
              size="sm"
              class="settings__mobile-back-btn"
              @click="goBack"
            >
              <template #leading>
                <UIcon name="material-symbols:arrow-back-rounded" />
              </template>
              Назад
            </UButton>
            <h3 class="settings__mobile-detail-title">{{ items[Number(active)]?.label }}</h3>
          </div>
          <div class="settings__mobile-detail-content">
            <UserSettingTab v-if="active === '0'"/>
          </div>
        </div>
      </Transition>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { TabsItem } from '@nuxt/ui'
import UserSettingTab from './UserSettingTab.vue'

const active = ref('0')
const isMobile = ref(false)
const showMobileDetail = ref(false)

const items = ref<(TabsItem & { iconColor?: string })[]>([
  {
    label: 'Общие',
    icon: 'mdi:user-outline',
    iconColor: 'var(--ui-primary)'
  },
  {
    label: 'Telegram',
    icon: 'ic:outline-telegram',
    iconColor: '#24A1DE'
  }
])

const selectTab = (index: string) => {
  active.value = index
  showMobileDetail.value = true
}

const goBack = () => {
  showMobileDetail.value = false
}

// Handle responsive viewports reactively on client side
if (import.meta.client) {
  const mediaQuery = window.matchMedia('(max-width: 767px)')
  isMobile.value = mediaQuery.matches

  const listener = (e: MediaQueryListEvent) => {
    isMobile.value = e.matches
    if (!e.matches) {
      // If we scale back to desktop view, exit detail screen state
      showMobileDetail.value = false
    }
  }

  onMounted(() => {
    mediaQuery.addEventListener('change', listener)
  })

  onUnmounted(() => {
    mediaQuery.removeEventListener('change', listener)
  })
}
</script>

<style lang="scss" scoped>
.settings {
  width: 100%;
  height: 100%;
  display: flex;
  gap: 2rem;

  &__tabs {
    width: 240px;
    height: 100%;
    flex-shrink: 0;
  }

  &__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
}

// Mobile specific styles
.settings__mobile {
  &-list {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  &-header {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  &-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--ui-text-highlighted);
    letter-spacing: -0.025em;
  }

  &-items {
    display: flex;
    flex-direction: column;
    background-color: var(--ui-bg);
    border: 1px solid var(--ui-border);
    border-radius: var(--ui-radius, 12px);
    overflow: hidden;
    box-shadow: var(--ui-shadow-sm, 0 1px 2px 0 rgba(0, 0, 0, 0.05));
  }

  &-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--ui-border);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.2s ease;
    width: 100%;
    outline: none;

    &:last-child {
      border-bottom: none;
    }

    &:active {
      background-color: var(--ui-bg-hovered, rgba(0, 0, 0, 0.04));
    }

    &-left {
      display: flex;
      align-items: center;
      gap: 1rem;
    }

    &-icon-wrapper {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 2.25rem;
      height: 2.25rem;
      background-color: transparent;
      transition: transform 0.2s ease;
    }

    &-icon {
      font-size: 1.5rem;
    }

    &-label {
      font-size: $text-sm;
      font-weight: 500;
      color: var(--ui-text-highlighted);
    }

    &-arrow {
      font-size: 1.25rem;
      color: var(--ui-text-muted);
      transition: transform 0.2s ease;
    }

    @media (hover: hover) {
      &:hover {
        background-color: var(--ui-bg-hovered, rgba(0, 0, 0, 0.02));

        .settings__mobile-item-icon-wrapper {
          transform: scale(1.1);
        }

        .settings__mobile-item-arrow {
          transform: translateX(2px);
          color: var(--ui-text-highlighted);
        }
      }
    }
  }

  &-detail {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;

    &-header {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding-bottom: 0.875rem;
      border-bottom: 1px solid var(--ui-border);
    }

    &-title {
      font-size: $text-sm;
      font-weight: 500;
      color: $text-muted;
      margin: 0 0 0 auto;
      line-height: 1;
      display: inline-flex;
      align-items: center;
    }

    &-content {
      width: 100%;
      display: flex;
    }
  }

  &-back-btn {
    padding-left: 0.25rem;
    padding-right: 0.5rem;
  }
}

// Slide fade transition
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(8px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}
</style>