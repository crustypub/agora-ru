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
import { useNavigation } from '~/composables/useNavigation';
import { useAuthUser } from '~/composables/useAuthUser';
import { computed } from 'vue';

const { navItems } = useNavigation();
const authUser = useAuthUser();

const handleLogout = () => {
    const token = useCookie('auth_token');
    token.value = null;
    authUser.value = null;
    navigateTo('/');
};

// Меню "Еще" для десктопного аватара
const moreItems = computed(() => {
    const items = [
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
</style>