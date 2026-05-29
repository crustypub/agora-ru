<template>
  <UCard>
    <template #header>
      <div class="article-skeleton__top">
        <USkeleton class="h-[18px] w-[45%]" />
        <div class="article-skeleton__stats">
          <USkeleton class="h-6 w-11" />
          <USkeleton class="h-6 w-11" />
        </div>
      </div>
    </template>

    <template #footer>
      <div class="article-skeleton__info">
        <div class="article-skeleton__user">
          <USkeleton class="h-3 w-[70px]" />
          <USkeleton class="h-5 w-5 rounded-full" />
        </div>
        <div class="article-skeleton__user">
          <USkeleton class="h-3 w-[70px]" />
          <USkeleton class="h-5 w-5 rounded-full" />
        </div>
      </div>
    </template>
  </UCard>
</template>

<script setup lang="ts"></script>

<style lang="scss" scoped>
.article-skeleton {
  &__top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  &__stats {
    display: flex;
    gap: 0.5rem;
  }

  &__info {
    display: flex;
    align-items: center;
    gap: 1.25rem;
  }

  &__user {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
}
</style>
