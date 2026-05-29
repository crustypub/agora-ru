<template>
    <UCard class="article-card" @click="navigateToArticle" :ui="{ root: 'cursor-pointer transition-all duration-200 hover:border-[var(--ui-primary)] hover:shadow-md' }">
        <template #header>
            <div class="article-card__top">
                <NuxtLink :to="`/wiki/${data.id}`" class="article-card__link" @click.stop>{{ data.title }}</NuxtLink>
                <div class="article-card__stats">
                    <UButton 
                        :icon="isStarred ? 'line-md:star-filled' : 'line-md:star'" 
                        size="xs" 
                        :color="isStarred ? 'primary' : 'neutral'" 
                        variant="ghost"
                        @click.stop.prevent="toggleStar"
                    >
                        {{ Number(starsCount) || 0 }}
                    </UButton>
                    <UButton 
                        icon="iconamoon:comment-fill" 
                        size="xs" 
                        color="neutral" 
                        variant="ghost"
                        @click.stop.prevent="navigateToComments"
                    >
                        {{ data.comment_count || 0 }}
                    </UButton>
                </div>
            </div>
        </template>

        <template #footer>
            <div class="article-card__info">
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
                <UBadge v-if="!!data.is_confirmed" color="success" variant="subtle" size="sm" class="article-status">
                    <UIcon name="material-symbols:check-circle-rounded" class="size-4" />
                    Подтверждено
                </UBadge>
                <UBadge v-else color="warning" variant="subtle" size="sm" class="article-status">
                    <UIcon name="material-symbols:info" class="size-4" />
                    На проверке
                </UBadge>
            </div>
        </template>
    </UCard>
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

const navigateToComments = () => {
    navigateTo(`/wiki/${props.data.id}#comments`);
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
.article-card {
    &__top {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    &__stats {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    &__link {
        font-size: $text-md;
        font-weight: 600;
        color: var(--ui-text-highlighted);
        transition: color 0.2s ease;
        line-height: 1.4;

        &:hover {
            color: var(--ui-primary);
        }
    }

    &__info {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 1.25rem;

        .article-status {
            margin-left: auto;
        }

        .user-wrapper {
            display: flex;
            align-items: center;
            gap: 0.5rem;

            &__title {
                font-size: $text-xs;
                color: var(--ui-text-muted);
                white-space: nowrap;
            }
        }
    }
}
</style>
