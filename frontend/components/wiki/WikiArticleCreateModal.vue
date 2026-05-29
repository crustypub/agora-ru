<template>
    <UModal fullscreen title="Создание Wiki-статьи" v-model:open="model">
        <template #body>
            <MdEditorWrapperClient :wiki-types="wikiTypes" :close-modal="closeModal" :submit="submit"/>
        </template>
    </UModal>
</template>

<script setup lang="ts">
import type { IWikiType } from '~/models/entities/wiki.entities';
import MdEditorWrapperClient from '../mdEditor/MdEditorWrapper.client.vue';
const model = defineModel<boolean>()


interface IProps {
    wikiTypes: IWikiType[] | undefined,
    submit: () => void,
}

const { wikiTypes, submit } = defineProps<IProps>();

const closeModal = function() {
    model.value = false;
}
</script>

<style lang="scss" scoped></style>
