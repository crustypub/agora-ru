export default defineAppConfig({
    ui: {
        colors: {
            primary: 'greek-blue',
            secondary: 'greek-olive',
            neutral: 'zinc',
        },
        header: {
            slots: {
                root: 'bg-primary border-b-0 h-14 shrink-0',
                container: 'flex items-center justify-between gap-3 h-full max-w-none px-4',
                left: 'lg:flex-1 flex items-center gap-1.5',
                center: 'hidden lg:flex',
                right: 'flex items-center justify-end lg:flex-1 gap-1.5',
                title: 'shrink-0 flex items-end gap-1.5 text-white',
                toggle: 'lg:hidden text-white',
            },
        },
        sidebar: {
            slots: {
                container: 'top-14 h-[calc(100vh-3.5rem)] bg-[var(--ui-bg)] border-r border-[var(--ui-border)]',
            }
        },
        footer: {
            slots: {
                root: 'bg-secondary border-t border-secondary-600',
                container: 'py-3 lg:py-3 lg:flex lg:items-center lg:justify-between lg:gap-x-3',
                left: 'flex items-center justify-center lg:justify-start lg:flex-1 gap-x-1.5 mt-2 lg:mt-0 lg:order-1',
                center: 'mt-2 lg:mt-0 lg:order-2 flex items-center justify-center',
                right: 'lg:flex-1 flex items-center justify-center lg:justify-end gap-x-1.5 lg:order-3',
            }
        },
        card: {
            slots: {
                root: 'rounded-none',
                header: 'rounded-none',
                body: 'rounded-none',
                footer: 'rounded-none',
            }
        },
        container: {
            base: 'max-w-[1150px]'
        },
        button: {
            base: 'cursor-pointer'
        },
        tabs: {
            slots: {
                trigger: 'cursor-pointer'
            }
        },
        select: {
            slots: {
                content: 'w-auto min-w-(--reka-select-trigger-width)',
                itemLabel: 'overflow-visible text-clip',
            }
        }
    }
})

