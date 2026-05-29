<template>
    <nav class="mobile-nav">
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

        <UDropdownMenu :items="moreItems" :popper="{ placement: 'top-end' }" class="mobile-nav__dropdown">
            <button class="mobile-nav__item mobile-nav__item--more">
                <UIcon name="material-symbols:more-horiz" class="mobile-nav__icon" />
                <span class="mobile-nav__label">Еще</span>
            </button>
        </UDropdownMenu>
    </nav>
</template>

<script setup lang="ts">
import { useNavigation } from '~/composables/useNavigation';
import { useAuthUser } from '~/composables/useAuthUser';
import { computed } from 'vue';

const { quickItems } = useNavigation();
const authUser = useAuthUser();

const handleLogout = () => {
    const token = useCookie('auth_token');
    token.value = null;
    authUser.value = null;
    navigateTo('/');
};

const moreItems = computed(() => {
    const items = [
        {
            label: 'О проекте',
            icon: 'material-symbols:info-outline',
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
.mobile-nav {
    display: none; // Скрыто на десктопе

    @media (max-width: 767.98px) {
        display: flex;
        align-items: stretch;
        justify-content: space-around;
        background-color: $bg-primary;
        border-top: 1px solid $border-color;
        width: 100%;
        height: 60px;
        flex-shrink: 0;
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
        transition: all 0.2s ease-in-out;
        flex: 1;
        height: 100%;
        cursor: pointer;
        background: transparent;
        border: none;
        outline: none;
        border-radius: 0; // Совершенно без закруглений

        &:hover {
            color: $text-primary;
        }

        &--active {
            background-color: $bg-dark;
            color: $white;

            &:hover {
                color: $white;
            }

            .mobile-nav__icon {
                color: $white;
            }
        }
    }

    &__dropdown {
        flex: 1;
        height: 100%;
        display: flex;
        
        :deep(> button),
        :deep(> div) {
            width: 100%;
            height: 100%;
            border-radius: 0;
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

    &__item:active &__icon {
        transform: scale(0.9);
    }
}
</style>
