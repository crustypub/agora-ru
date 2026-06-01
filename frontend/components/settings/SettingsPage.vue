<template>
  <div class="settings">
    <div class="settings__tabs">
      <UTabs v-model="active" orientation="vertical" size="md" variant="link" :items="items" class="w-full" />
    </div>
    <div class="settings__content">
      <UserSettingTab v-if="active === '0'"/>

    </div>
  </div>
</template>
<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
import UserSettingTab from './UserSettingTab.vue';
const active = ref('0');

const items = ref<TabsItem[]>([
  {
    label: 'Общие',
    icon: 'mdi:user-outline'
  },
  {
    label: 'Telegram',
    icon: 'ic:outline-telegram',
  }
])


</script>

<style lang="scss" scoped>
.settings {
  width: 100%;
  height: 100%;
  display: flex;
  column-gap: 1fr;

  &__tabs {
    width: auto;
    height: 100%;
  }

  &__content {
    flex: 1;
    display: flex;
  }
}
</style>