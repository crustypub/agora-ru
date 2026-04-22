<template>
    <div class="container">
        {{ data.title }}
    </div>    
</template>
<script setup lang="ts">
import type { IPostResponseItem } from '~/models/entities/post.entities';

interface IProps {
    data: IPostResponseItem
}

const { data } = defineProps<IProps>();

</script>

<style lang="scss" scoped>

.container {
    min-width: 100%;
    height: 2rem;
    display: flex;
    flex-direction: column;
    padding: .5rem;
    background-color: beige;
}

</style>