<template>
    <UFooter>
        <template #left>
            <p class="copyright">Copyright © 2026 Agora RU</p>
        </template>

        <template #default>
            <UNavigationMenu :items="navItems" variant="link" />
        </template>

        <template #right>
            <UButton
                icon="i-simple-icons-telegram"
                color="neutral"
                variant="ghost"
                to="https://t.me/"
                target="_blank"
                aria-label="Telegram"
            />
        </template>
    </UFooter>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';

const navItems: NavigationMenuItem[] = [
    {
        label: 'Главная',
        to: '/',
    },
    {
        label: 'Авторизация',
        to: '/auth',
    },
]
</script>

<style lang="scss" scoped>
.copyright {
    font-size: $text-sm;
    color: $white;
    opacity: 0.8;
}
</style>