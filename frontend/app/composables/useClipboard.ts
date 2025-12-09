export function useClipboard() {
    const isSupported = computed(() =>
        typeof navigator !== 'undefined' &&
        typeof window !== 'undefined' &&
        !!navigator.clipboard &&
        !!navigator.clipboard.writeText
    );

    const copying = ref(false);
    const lastCopied = ref<string | null>(null);

    const copy = async (text: string | null | undefined): Promise<boolean> => {
        if (!text) return false;
        copying.value = true;

        try {
            if (isSupported.value && window.isSecureContext) {
                await navigator.clipboard.writeText(text);
            } else {
                const textarea = document.createElement('textarea');
                textarea.value = text;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
            }
            lastCopied.value = text;
            return true;
        } catch {
            return false;
        } finally {
            copying.value = false;
        }
    };

    return {
        copy,
        isSupported,
        copying,
        lastCopied,
    };
}