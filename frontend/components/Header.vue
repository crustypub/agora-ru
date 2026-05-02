<template>
    <UHeader mode="slideover"
        :toggle="{ color: 'primary', variant: 'solid', class: 'text-white hover:bg-primary-600/50 data-[state=open]:bg-primary-600/50' }">
        <template #left>
            <NuxtLink to="/" class="main-link">
                <span class="main-link__content">Agora Ru</span>
            </NuxtLink>
        </template>

        <UNavigationMenu :items="navItems" variant="link" :ui="{
            root: 'gap-0',
            link: [
                // Цвет текста
                'text-white/60 hover:text-white data-[active=true]:text-white',
                // Типографика
                'font-medium text-sm tracking-wide',
                // Переходы
                'transition-colors duration-200',
                // Анимированное подчёркивание активного пункта
                'relative',
                'after:absolute after:bottom-0 after:inset-x-2.5',
                'after:h-px after:rounded-full after:bg-white/80',
                'after:scale-x-0 data-[active=true]:after:scale-x-100',
                'after:transition-transform after:duration-300 after:ease-out after:origin-center',
            ].join(' '),
            linkLeadingIcon: [
                'size-4 text-white/50',
                'group-hover:text-white/90 data-[active=true]:text-white',
                'transition-colors duration-200',
            ].join(' '),
        }" />

        <template #body>
            <UNavigationMenu :items="navItems" orientation="vertical" class="-mx-2.5" />
        </template>
    </UHeader>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';

const route = useRoute()


const navItems = computed<NavigationMenuItem[]>(() => [
    {
        label: 'Главная',
        to: '/',
        icon: 'ic:outline-house',
        active: route.path === '/',
    },
    {
        label: 'Wiki',
        to: '/wiki',
        icon: 'material-symbols:book-ribbon-outline',
        active: route.path.startsWith('/wiki'),
    },
    {
        label: 'Объявления',
        to: '/discussions',
        icon: 'ic:outline-shopping-basket',
        active: route.path.startsWith('/discussions'),
    },
    {
        label: 'О проекте',
        to: '/about',
        icon: 'ic:outline-info',
        active: route.path === '/about',
    },
])
</script>

<style lang="scss" scoped>
.main-link,
.main-link:visited,
.main-link:hover,
.main-link:active {
    all: unset;
    cursor: pointer;
}

.main-link__content {
    font-size: $text-2xl;
    font-family: 'Caesar Dressing', sans-serif;
    color: $white;
}

// Кнопка авторизации — белый outline на тёмном фоне
.auth-btn {
    --ui-border: rgba(255, 255, 255, 0.35);
    --ui-text: rgba(255, 255, 255, 0.85);

    &:hover {
        --ui-border: rgba(255, 255, 255, 0.7);
        --ui-text: #fff;
        --ui-bg: rgba(255, 255, 255, 0.08);
    }
}

.user-avatar {
    cursor: pointer;
    opacity: 0.9;
    transition: opacity 0.2s ease;

    &:hover {
        opacity: 1;
    }
}
</style>