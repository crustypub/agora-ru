import { computed } from 'vue';
import { useRoute } from '#app';

export const useNavigation = () => {
    const route = useRoute();

    const navItems = computed(() => [
        {
            label: 'Главная',
            to: '/',
            icon: 'ic:outline-house',
            active: route.path === '/',
        },
        {
            label: 'Wiki',
            to: '/wiki',
            icon: 'material-symbols:book-ribbon-outline',
            active: route.path.startsWith('/wiki'),
        },
        {
            label: 'Объявления',
            to: '/discussions',
            icon: 'ic:outline-shopping-basket',
            active: route.path.startsWith('/discussions'),
        },
        {
            label: 'О проекте',
            to: '/about',
            icon: 'ic:outline-info',
            active: route.path === '/about',
        },
    ]);

    const quickItems = computed(() => [
        {
            label: 'Главная',
            to: '/',
            icon: 'ic:outline-house',
            active: route.path === '/',
        },
        {
            label: 'Wiki',
            to: '/wiki',
            icon: 'material-symbols:book-ribbon-outline',
            active: route.path.startsWith('/wiki'),
        },
        {
            label: 'Объявления',
            to: '/discussions',
            icon: 'ic:outline-shopping-basket',
            active: route.path.startsWith('/discussions'),
        },
    ]);

    return {
        navItems,
        quickItems,
    };
};
