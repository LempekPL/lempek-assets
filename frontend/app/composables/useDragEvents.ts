export function useDragEvents() {
    const isFileDrag = (e: DragEvent): boolean => {
        const types = e.dataTransfer?.types;
        if (!types) return false;
        return Array.from(types).includes("Files");
    };

    return {
        isFileDrag
    };
}