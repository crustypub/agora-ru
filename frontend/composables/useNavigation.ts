import { computed } from 'vue';
import { useRoute } from '#app';
import { useChat } from './useChat';

export const useNavigation = () => {
    const route = useRoute();
    const { totalUnreadCount } = useChat();

    const navItems = computed(() => [
        {
            label: 'Главная',
            to: '/',
            icon: 'material-symbols:home-outline',
            active: route.path === '/',
        },
        {
            label: 'Пользователи',
            to: '/users',
            icon: 'material-symbols:group-outline',
            active: route.path.startsWith('/users'),
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
            icon: 'material-symbols:shopping-basket-outline',
            active: route.path.startsWith('/discussions'),
        },
        {
            label: 'Чаты',
            to: '/chats',
            icon: 'material-symbols:chat-bubble-outline',
            active: route.path.startsWith('/chats'),
            badge: totalUnreadCount.value > 0 ? totalUnreadCount.value : undefined
        },
        {
            label: 'О проекте',
            to: '/about',
            icon: 'material-symbols:info-outline',
            active: route.path === '/about',
        },
    ]);

    const quickItems = computed(() => [
        {
            label: 'Главная',
            to: '/',
            icon: 'material-symbols:home-outline',
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
            icon: 'material-symbols:shopping-basket-outline',
            active: route.path.startsWith('/discussions'),
        },
        {
            label: 'Чаты',
            to: '/chats',
            icon: 'material-symbols:chat-bubble-outline',
            active: route.path.startsWith('/chats'),
        },
    ]);

    return {
        navItems,
        quickItems,
    };
};

