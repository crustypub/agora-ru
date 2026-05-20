<template>
    <div class="article" @click="navigateToArticle">
        <div class="article__top">
            <NuxtLink :to="`/wiki/${data.id}`" class="article__link" @click.stop>{{ data.title }}</NuxtLink>
            <UButton 
                :icon="isStarred ? 'line-md:star-filled' : 'line-md:star'" 
                size="xs" 
                :color="isStarred ? 'primary' : 'neutral'" 
                variant="ghost"
                @click.stop.prevent="toggleStar"
            >
                {{ Number(starsCount) || 0 }}
            </UButton>
        </div>
        <div class="article__info">
            <div class="user-wrapper">
                <span class="user-wrapper__title">Автор: </span>
                <UTooltip :text="`${data?.created_by?.first_name}  @${data?.created_by?.username}`" :delay-duration="0">
                    <UAvatar :src="data.created_by.avatar_url || ''" :alt="data?.created_by?.first_name || '.'"
                        size="xs" />
                </UTooltip>
            </div>
            <div class="user-wrapper">
                <span class="user-wrapper__title">Последняя редакция: </span>
                <UTooltip :text="`${data?.last_edited_by?.first_name}  @${data?.last_edited_by?.username}`"
                    :delay-duration="0">
                    <UAvatar :src="data.last_edited_by.avatar_url || ''" :alt="data?.last_edited_by?.first_name || '.'"
                        size="xs" />
                </UTooltip>
            </div>
            <UTooltip text="Статья подтверждена" :delay-duration="0" class="article-status" v-if="!!data.is_confirmed">
                <UIcon name="material-symbols:check-circle-rounded" class="text-success-500 size-5" />
            </UTooltip>
            <UTooltip text="Статья не подтверждена" :delay-duration="0" class="article-status" v-else-if="!data.is_confirmed">
                <UIcon name="material-symbols:info" class="text-error-500 size-5" />
            </UTooltip>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { IWikiArticleSimple } from '~/models/entities/wiki.entities';
import { useApiCall } from '~/composables/useApi';

interface IProps {
    data: IWikiArticleSimple
}

const props = defineProps<IProps>();

const isStarred = ref(props.data.is_starred);
const starsCount = ref(props.data.stars_count);

watch(() => props.data, (newVal) => {
    isStarred.value = newVal.is_starred;
    starsCount.value = newVal.stars_count;
}, { deep: true });

const navigateToArticle = () => {
    navigateTo(`/wiki/${props.data.id}`);
};

const toggleStar = async () => {
    const previousIsStarred = isStarred.value;
    const previousStarsCount = starsCount.value;

    isStarred.value = !previousIsStarred;
    starsCount.value = previousIsStarred ? previousStarsCount - 1 : previousStarsCount + 1;

    try {
        if (previousIsStarred) {
            await useApiCall(`/api/wiki/${props.data.id}/star`, { method: 'DELETE' });
        } else {
            await useApiCall(`/api/wiki/${props.data.id}/star`, { method: 'PATCH' });
        }
    } catch (e) {
        isStarred.value = previousIsStarred;
        starsCount.value = previousStarsCount;
        console.error('Failed to toggle star:', e);
    }
};

</script>

<style lang="scss" scoped>
.article {
    display: flex;
    flex-direction: column;
    row-gap: 0.75rem;
    width: 100%;
    padding: 1rem 1.25rem;
    background-color: $bg-primary;
    border: 1px solid $border-color;
    border-radius: 8px;
    text-decoration: none;
    color: inherit;
    transition: all 0.2s ease-in-out;
    box-shadow: 0 2px 4px rgba($black, 0.02);
    cursor: pointer;

    &:hover {
        box-shadow: 0 6px 12px rgba($black, 0.08);
        border-color: $primary;

        .article__link {
            color: $primary;
        }
    }

    &__top {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 1rem;
    }

    &__link {
        font-size: $text-md;
        font-weight: 600;
        color: $text-primary;
        transition: color 0.2s ease;
        line-height: 1.4;
    }

    &__info {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 1.25rem;
        padding-top: 0.75rem;
        border-top: 1px solid $gray-200;

        .article-status {
            margin-left: auto;
        }

        .user-wrapper {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            &__title {
                font-size: $text-xs;
                color: $text-muted;
                white-space: nowrap;
            }
        }
    }
}
</style>
