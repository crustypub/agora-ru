<template>
    <UHeader :toggle="false">
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

        <template #right>
            <div class="header__auth">
                <template v-if="authUser">
                    <UDropdownMenu :items="moreItems" :popper="{ placement: 'bottom-end' }">
                        <UAvatar
                            :src="authUser.avatar_url || ''"
                            :alt="authUser.first_name || authUser.username || ''"
                            class="user-avatar"
                            size="sm"
                        />
                    </UDropdownMenu>
                </template>
                <template v-else>
                    <UButton to="/auth" class="auth-btn" variant="outline" size="sm">
                        Войти
                    </UButton>
                </template>
            </div>
        </template>
    </UHeader>

    <!-- Мобильный навигационный бар (как в iOS/Android аппах) -->
    <nav class="mobile-nav">
        <div class="mobile-nav__bar">
            <NuxtLink
                v-for="item in quickItems"
                :key="item.to"
                :to="item.to"
                class="mobile-nav__item"
                :class="{ 'mobile-nav__item--active': item.active }"
            >
                <UIcon :name="item.icon" class="mobile-nav__icon" />
                <span class="mobile-nav__label">{{ item.label }}</span>
            </NuxtLink>

            <UDropdownMenu :items="moreItems" :popper="{ placement: 'top-end' }">
                <button class="mobile-nav__item mobile-nav__item--more">
                    <UIcon name="material-symbols:more-horiz" class="mobile-nav__icon" />
                    <span class="mobile-nav__label">Еще</span>
                </button>
            </UDropdownMenu>
        </div>
    </nav>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui';
import { useAuthUser } from '~/composables/useAuthUser';
import { computed } from 'vue';

const route = useRoute();
const authUser = useAuthUser();

const handleLogout = () => {
    const token = useCookie('auth_token');
    token.value = null;
    authUser.value = null;
    navigateTo('/');
};

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
]);

// Быстрые ссылки для мобильного навбара
const quickItems = computed(() => [
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
]);

// Меню "Еще" для мобильного навбара
const moreItems = computed(() => {
    const items: any = [
        {
            label: 'О проекте',
            icon: 'ic:outline-info',
            to: '/about',
        },
    ];

    if (authUser.value) {
        items.push({
            label: `Профиль: ${authUser.value.first_name || authUser.value.username}`,
            icon: 'material-symbols:account-circle-outline',
            disabled: true,
        });
        items.push({
            label: 'Выйти',
            icon: 'material-symbols:logout-rounded',
            onSelect: () => handleLogout(),
        });
    } else {
        items.push({
            label: 'Войти',
            icon: 'material-symbols:login-rounded',
            to: '/auth',
        });
    }

    return [items];
});
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

.header__auth {
    display: flex;
    align-items: center;
}

// ========== МОБИЛЬНЫЙ НАВБАР ==========
.mobile-nav {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    width: calc(100% - 2rem);
    max-width: 440px;
    z-index: 1000;
    display: none; // Скрыт на десктопе

    @media (max-width: 767.98px) {
        display: block;
    }

    &__bar {
        display: flex;
        justify-content: space-around;
        align-items: center;
        background-color: rgba($bg-primary, 0.95);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba($border-color, 0.7);
        border-radius: 9999px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.08), 0 2px 5px rgba(0, 0, 0, 0.03);
        padding: 0.4rem 0.5rem;
    }

    &__item {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
        text-decoration: none;
        color: $text-secondary;
        font-family: inherit;
        border-radius: 9999px;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        padding: 0.4rem 0.8rem;
        min-width: 70px;
        cursor: pointer;
        background: transparent;
        border: none;
        outline: none;

        &:hover {
            color: $text-primary;
        }

        &--active {
            background-color: $bg-dark;
            color: $white;
            padding: 0.4rem 1rem;

            &:hover {
                color: $white;
            }

            .mobile-nav__icon {
                color: $white;
            }
        }
    }

    &__icon {
        font-size: 1.35rem;
        transition: transform 0.2s ease;
    }

    &__label {
        font-size: 0.7rem;
        font-weight: 500;
    }

    // Микро-анимация при нажатии
    &__item:active &__icon {
        transform: scale(0.85);
    }
}

// Глобальный отступ снизу на мобильных устройствах, чтобы контент не перекрывался
:global(.layout-main) {
    @media (max-width: 767.98px) {
        padding-bottom: 6.5rem !important;
    }
}
</style>