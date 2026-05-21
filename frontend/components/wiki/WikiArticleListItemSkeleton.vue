<template>
  <div class="article-skeleton">
    <div class="article-skeleton__top">
      <USkeleton class="article-skeleton__title" />
      <div class="article-skeleton__stats">
        <USkeleton class="article-skeleton__stat-btn" />
        <USkeleton class="article-skeleton__stat-btn" />
      </div>
    </div>
    <div class="article-skeleton__info">
      <div class="article-skeleton__user">
        <USkeleton class="article-skeleton__label" />
        <USkeleton class="article-skeleton__avatar" />
      </div>
      <div class="article-skeleton__user">
        <USkeleton class="article-skeleton__label" />
        <USkeleton class="article-skeleton__avatar" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts"></script>

<style lang="scss" scoped>
.article-skeleton {
  display: flex;
  flex-direction: column;
  row-gap: 0.75rem;
  width: 100%;
  padding: 1rem 1.25rem;
  background-color: $bg-primary;
  border: 1px solid $border-color;
  border-radius: 8px;

  &__top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  &__title {
    height: 18px;
    width: 45%;
    border-radius: 4px;
  }

  &__stats {
    display: flex;
    gap: 0.5rem;
  }

  &__stat-btn {
    height: 24px;
    width: 44px;
    border-radius: 6px;
  }

  &__info {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding-top: 0.75rem;
    border-top: 1px solid $gray-200;
  }

  &__user {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  &__label {
    height: 12px;
    width: 70px;
    border-radius: 4px;
  }

  &__avatar {
    height: 20px;
    width: 20px;
    border-radius: 50%;
  }
}
</style>
