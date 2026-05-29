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
    </UHeader>
</template>

<script setup lang="ts">
const isSidebarOpen = useState('isSidebarOpen');
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