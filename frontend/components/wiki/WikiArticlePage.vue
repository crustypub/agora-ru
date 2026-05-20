<template>
    <div class="wiki-container">
        <div v-if="!!response?.data?.content" class="wiki-content">
            <MdPreview editorId="preview-only" :modelValue="response?.data?.content" language="en-US" />
        </div>
        <CommentSection v-if="articleIdStr" entity-type="wiki" :entity-id="articleIdStr" />
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { MdPreview } from 'md-editor-v3';
import type { IWikiArticleResponse } from '~/models/api/wiki.api';
import CommentSection from '../comments/CommentSection.vue';

interface IProps {
    articleId: string | string[] | undefined;
}

const props = defineProps<IProps>();

const articleIdStr = computed(() => {
    if (Array.isArray(props.articleId)) return props.articleId[0];
    return props.articleId as string;
});

const { data: response } = await useApi<IWikiArticleResponse>(`/api/wiki/${articleIdStr.value}`);
</script>

<style lang="scss" scoped>
.wiki-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    max-width: 900px;
    margin: 0 auto;
    padding-bottom: 4rem;
}

.wiki-content {
    background-color: $bg-primary;
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid $border-color;
    margin-bottom: 2rem;
}

@media (max-width: 768px) {
    .wiki-container {
        padding: 0 1rem 4rem;
    }
    .wiki-content {
        padding: 0.5rem;
    }
}
</style>