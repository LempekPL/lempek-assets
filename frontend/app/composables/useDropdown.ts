export function useDropdown(
    closeAfterClick: boolean = false,
    {openCB, closeCB, outsideClickCB}: {
        openCB: VoidFunction | undefined,
        closeCB: VoidFunction | undefined,
        outsideClickCB: VoidFunction | undefined
    } = {openCB: undefined, closeCB: undefined, outsideClickCB: undefined,}
) {
    const show = ref<boolean>(false);
    const buttonRef = ref<HTMLElement | null>(null);
    const dropdownRef = ref<HTMLElement | null>(null);

    function toggle() {
        show.value = !show.value;
        if (show.value) {
            if (openCB) openCB();
        } else {
            if (closeCB) closeCB();
        }
    }

    function open() {
        show.value = true;
        if (openCB) openCB();
    }

    function close() {
        show.value = false;
        if (closeCB) closeCB();
    }

    function handleClickOutside(event: MouseEvent) {
        if (
            (closeAfterClick || !dropdownRef.value?.contains(event.target as Node)) &&
            !buttonRef.value?.contains(event.target as Node)
        ) {
            if (show.value && outsideClickCB)
                outsideClickCB();
            close()
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            if (show.value && outsideClickCB)
                outsideClickCB();
            close();
        }
    }

    onMounted(() => {
        window.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('keydown', handleKeydown);
    });
    onBeforeUnmount(() => {
        window.removeEventListener('mousedown', handleClickOutside);
        window.removeEventListener('keydown', handleKeydown);
    });

    return {
        show,
        buttonRef,
        dropdownRef,
        toggle,
        open,
        close,
    };
}