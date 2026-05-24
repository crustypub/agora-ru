export default defineNuxtPlugin((nuxtApp) => {
    nuxtApp.hook('page:finish', () => {
        const main = document.querySelector('.layout-main');
        if (main) {
            main.scrollTop = 0;
        }
    });
});
