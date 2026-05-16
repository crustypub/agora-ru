<template>
    <div class="wiki-container">
        <div v-if="!!response?.data?.content">
            <MdPreview editorId="preview-only" :modelValue="response?.data?.content" language="en-US" />
        </div>
    </div>
</template>
<script setup lang="ts">
import { MdPreview } from 'md-editor-v3';
import type { IWikiArticleResponse, IWikiTypesResponse } from '~/models/api/wiki.api';


interface IProps {
    articleId: string | string[] | undefined;
}

const { articleId } = defineProps<IProps>();

const { data: response } = await useApi<IWikiArticleResponse>(`/api/wiki/${articleId}`);

</script>

<style lang="scss" scoped>
.wiki-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
}
</style>