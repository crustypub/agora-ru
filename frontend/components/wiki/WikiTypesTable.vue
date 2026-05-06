<template>
    <div class="wiki-types">
        <template v-if="Array.isArray(data)">
            <NuxtLink v-for="item in data" :key="item.id" :to="`/wiki/type/${item.id}`"
                class="wiki-types__card">
                <UIcon :name="getWikiTypeIcon(item.title)" class="wiki-types__icon" />
                <span class="wiki-types__title">{{ item.title }}</span>
                <UIcon name="material-symbols:arrow-forward" class="wiki-types__arrow" />
            </NuxtLink>
        </template>
    </div>
</template>

<script setup lang="ts">
import type { IWikiTypeResponseItem } from '~/models/entities/wiki.entities';


interface Props {
    data: IWikiTypeResponseItem[] | undefined,
}

const { data } = defineProps<Props>();


function getWikiTypeIcon(title: string): string {
    switch (title) {
        case 'Легализация и документы':
            return 'material-symbols:badge';
        case 'Жильё и быт':
            return 'material-symbols:home';
        case 'Медицина и страхование':
            return 'material-symbols:medical-services';
        case 'Работа и налоги':
            return 'material-symbols:work';
        case 'Безопасность и чрезвычайные ситуации':
            return 'material-symbols:emergency';
        case 'Другое':
            return 'material-symbols:help-outline';
        default:
            return 'material-symbols:help-outline';
    }
}
</script>

<style lang="scss" scoped>
.wiki-types {
    width: 100%;
    max-height: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
    padding: 1rem 0;

    &__card {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
        padding: 24px 20px 20px;
        background-color: $sand;
        border-radius: 8px;
        text-decoration: none;
        color: $text-primary;
        transition: background-color 0.2s ease, transform 0.2s ease;

        &:hover {
            background-color: darken($sand, 6%);
            transform: translateY(-2px);

            .wiki-types__arrow {
                transform: translateX(4px);
            }
        }
    }

    &__icon {
        width: 2rem;
        height: 2rem;
        color: $blue-dark;
    }

    &__title {
        flex: 1;
        font-size: $text-md;
        font-weight: 500;
        line-height: 1.3;
    }

    &__arrow {
        align-self: flex-end;
        width: 1.25rem;
        height: 1.25rem;
        color: $text-muted;
        transition: transform 0.2s ease;
    }
}
</style>