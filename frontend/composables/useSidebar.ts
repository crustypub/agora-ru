import { watch } from 'vue';

export const useSidebar = () => {
    const cookie = useCookie<boolean>('sidebar_open', { default: () => false });
    const isSidebarOpen = useState<boolean>('isSidebarOpen', () => cookie.value);

    // Sync state changes to cookie
    watch(isSidebarOpen, (newVal) => {
        cookie.value = newVal;
    });

    return isSidebarOpen;
};
