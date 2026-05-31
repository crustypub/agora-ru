<template>
    <div class="user-form">
        <UForm :schema="schema" :state="state" class="md-editor-form" @submit="handleSubmit">
            <UFormField label="Username" name="username" required class="md-editor-form__row-field">
                <UInput v-model="state.username" placeholder="Введите username" class="w-full" />
            </UFormField>
            <UFormField label="Имя" name="firstname" required class="md-editor-form__row-field">
                <UInput v-model="state.firstname" placeholder="Введите имя" class="w-full" />
            </UFormField>

            <UFormField label="Фамилия" name="lastname" required class="md-editor-form__row-field">
                <UInput :model-value="state.lastname ?? ''"
                    @update:model-value="value => state.lastname = value || null" placeholder="Введите фамилию"
                    class="w-full" />
            </UFormField>


            <UButton type="submit" class="md-editor-form__submit">
                Сохранить
            </UButton>

        </UForm>
    </div>
</template>

<script setup lang="ts">
import { z } from 'zod';
import { removeEmptyStrings } from '~/helpers/common';
import type { IAuthMeValue } from '~/models/api/auth.api';
import type { IUserUpdate } from '~/models/api/user.api';

const { data: response } = await useApi<IAuthMeValue>('/api/auth/me');
const authUser = useAuthUser();

const schema = z.object({
    username: z.string({ message: 'Введите username' })
        .min(1, 'Введите username')
        .regex(/^[a-zA-Z0-9_]+$/, 'Username может содержать только латинские буквы, цифры и нижнее подчеркивание'),
    firstname: z.string({ message: 'Введите имя' })
        .min(1, 'Введите имя'),
    lastname: z.string({ message: 'Введите фамилию' })
        .min(1, 'Введите фамилию').nullish(),
});


interface IProps {

}

const props = defineProps<IProps>();

const defaultValue = {
    username: response?.value?.data.username || '',
    firstname: response?.value?.data.first_name || '',
    lastname: response?.value?.data.last_name || null,
}


const state = reactive(defaultValue);

const handleSubmit = async function () {
    try {
        console.log('removeEmptyStrings(state)', removeEmptyStrings(state), state);
        const response = await useApiCall<IUserUpdate>('/api/user', {
            method: 'PATCH',
            body: removeEmptyStrings(state),
        });

        if (response.status === 'success') {
            authUser.value = response.data;
        }

    } catch (e) { }
}


</script>

<style lang="scss" scoped>
.user-form {
    flex: 1;
}
</style>
