export default defineNuxtPlugin((nuxtApp) => {
    nuxtApp.hook('page:finish', () => {
        const scrollContainers = document.querySelectorAll('.layout-main, .layout-main__container');
        scrollContainers.forEach((el) => {
            if (el) {
                el.scrollTop = 0;
            }
        });
    });
});
