<template>
    <UModal fullscreen v-model:open="isOpenWikiCreateModal"">

        <template #body>
            <Placeholder class=" h-48" />
    <MdEditorWrapperClient :wiki-types="response?.data" />
</template>
</UModal>
<div class="wikitypes-container">
    <div class="wiki-create">
        <ClientOnly>
            <UButton icon="material-symbols:add-circle-outline-rounded" size="lg" color="primary" variant="soft"
                @click="openModal()">Создать
                статью</UButton>
        </ClientOnly>
    </div>
    <WikiTypesTable :data="response?.data" />
</div>
</template>
<script setup lang="ts">
import type { IWikiTypesResponse } from '~/models/api/wiki.api';
import MdEditorWrapperClient from '../mdEditor/MdEditorWrapper.client.vue';
import WikiTypesTable from './WikiTypesTable.vue';

const { data: response } = await useApi<IWikiTypesResponse>('/api/wiki_types');
const isOpenWikiCreateModal = ref(false);

const openModal = () => isOpenWikiCreateModal.value = true;

</script>

<style lang="scss" scoped>
.wikitypes-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
}

.wiki-create {
    width: 100%;
    min-height: 36px;
    height: auto;
    display: flex;
    justify-content: flex-end;
    margin-top: .5rem;
}
</style>