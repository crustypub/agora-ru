<template>
    <div class="container">
        <NuxtLink to="/" active-class="" exact-active-class="" class="main-link">
            <span class="main-link__content">
                Agora Ru
            </span>
        </NuxtLink>
    </div>
</template>
<script lang="ts"></script>

<style lang="scss" scoped>
.container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    padding: 0.25rem 1rem;
}



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
    font-family: $header-main-font;
}

</style>