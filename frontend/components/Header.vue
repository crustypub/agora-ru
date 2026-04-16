<template>
    <UHeader class="bg-primary w-full p-12px" >
        <template #left>
            <NuxtLink to="/" active-class="" exact-active-class="" class="main-link">
                <span class="main-link__content">
                    Agora Ru
                </span>
            </NuxtLink>
        </template>
    </UHeader>
</template>
<script setup lang="ts"></script>

<style lang="scss" scoped>

.main-link,
.main-link:visited,
.main-link:hover,
.main-link:active,
.main-link.nuxt-link-active,
.main-link.nuxt-link-exact-active {
    all: unset;
    /* Полный сброс всех CSS-свойств */
    cursor: pointer;
    /* Добавьте свои базовые стили здесь */
    color: $white;
}

.main-link__content {
    font-size: $text-2xl;
    font-family: 'Caesar Dressing', sans-serif;
}
</style>