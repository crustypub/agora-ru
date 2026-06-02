import { watch, nextTick } from 'vue';
import { useRoute } from '#app';

export default defineNuxtPlugin((nuxtApp) => {
    const route = useRoute();

    const handleScroll = (hash: string, smooth: boolean = false) => {
        nextTick(() => {
            const scrollContainers = document.querySelectorAll('.layout-main, .layout-main__container');
            
            if (hash) {
                const targetEl = document.querySelector(hash);
                if (targetEl) {
                    targetEl.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto' });
                    return;
                }
                
                // Fallback: reset scrollTop of scroll containers first
                scrollContainers.forEach((el) => {
                    if (el) {
                        el.scrollTop = 0;
                    }
                });

                // Retry after a short timeout if element wasn't immediately found due to layout delays
                setTimeout(() => {
                    const retryEl = document.querySelector(hash);
                    if (retryEl) {
                        retryEl.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto' });
                    }
                }, 100);
            } else {
                scrollContainers.forEach((el) => {
                    if (el) {
                        el.scrollTop = 0;
                    }
                });
            }
        });
    };

    nuxtApp.hook('page:finish', () => {
        handleScroll(route.hash, false);
    });

    watch(
        () => ({ path: route.path, hash: route.hash }),
        (newVal, oldVal) => {
            if (oldVal && newVal.path === oldVal.path && newVal.hash !== oldVal.hash) {
                if (newVal.hash) {
                    handleScroll(newVal.hash, true);
                }
            }
        }
    );
});
