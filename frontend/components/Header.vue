<template>
    <UHeader :toggle="false">
        <template #left>
            <!-- Custom dynamic animated burger button -->
            <button
                class="sidebar-toggle-btn"
                :class="{ 'is-active': isSidebarOpen }"
                @click="isSidebarOpen = !isSidebarOpen"
                aria-label="Toggle sidebar"
            >
                <div class="burger-icon">
                    <span class="line line-1"></span>
                    <span class="line line-2"></span>
                    <span class="line line-3"></span>
                </div>
            </button>
            
            <!-- Logo shown on both desktop and mobile -->
            <NuxtLink to="/" class="main-link">
                <span class="main-link__content">Agora Ru</span>
            </NuxtLink>
        </template>

        <!-- No center navigation menu, links are in the sidebar -->

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
</template>

<script setup lang="ts">
import { useAuthUser } from '~/composables/useAuthUser';
import { computed } from 'vue';

const isSidebarOpen = useState('isSidebarOpen');
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

.sidebar-toggle-btn {
    width: 38px;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    border: none;
    background-color: transparent;
    cursor: pointer;
    margin-right: 0.5rem;
    transition: background-color 0.2s ease, transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    outline: none;

    &:hover {
        background-color: rgba(255, 255, 255, 0.12);
        
        .line {
            background-color: #ffffff;
        }
    }

    &:focus-visible {
        outline: 2px solid rgba(255, 255, 255, 0.4);
        outline-offset: 2px;
    }

    .burger-icon {
        width: 18px;
        height: 12px;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .line {
        display: block;
        width: 100%;
        height: 2px;
        background-color: rgba(255, 255, 255, 0.9);
        border-radius: 9px;
        transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
        transform-origin: center;
    }

    &.is-active {
        transform: rotate(-180deg);

        .line-1 {
            transform: translateY(5px) rotate(45deg);
        }
        .line-2 {
            opacity: 0;
            transform: scaleX(0);
        }
        .line-3 {
            transform: translateY(-5px) rotate(-45deg);
        }
    }
}
</style>