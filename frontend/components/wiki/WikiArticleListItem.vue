<template>
    <NuxtLink class="article" :to="`wiki/${data.id}`">
        <div class="article__top">
            <span class="article__link">{{ data.title }}</span>
            <UButton icon="line-md:star" size="xs" color="neutral" variant="ghost">{{
                Number(data.stars_count) || 0 }}</UButton>
        </div>
        <div class="article__info">
            <div class="user-wrapper">
                <span class="user-wrapper__title">Автор: </span>
                <UTooltip :text="`${data?.created_by?.first_name}  @${data?.created_by?.username}`">
                    <UAvatar :src="data.created_by.avatar_url || ''" :alt="data?.created_by?.first_name || '.'"
                        size="xs" />
                </UTooltip>
            </div>
            <div class="user-wrapper">
                <span class="user-wrapper__title">Последняя редакция: </span>
                <UTooltip :text="`${data?.last_edited_by?.first_name}  @${data?.last_edited_by?.username}`">
                    <UAvatar :src="data.last_edited_by.avatar_url || ''" :alt="data?.last_edited_by?.first_name || '.'"
                        size="xs" />
                </UTooltip>
            </div>
        </div>
    </NuxtLink>
</template>

<script setup lang="ts">
import type { IWikiArticleSimple } from '~/models/entities/wiki.entities';

interface IProps {
    data: IWikiArticleSimple
}


const { data } = defineProps<IProps>();

</script>

<style lang="scss" scoped>
.article {
    width: 100%;
    height: auto;
    padding: 0.5rem 1rem;
    background-color: $gray-200;

    &__top {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    &__info {
        width: 100%;
        overflow-x: hidden;
        display: flex;
        column-gap: 12px;

        .user-wrapper {
            width: auto;

            &__title {
                white-space: nowrap;
            }
        }
    }

}
</style>
