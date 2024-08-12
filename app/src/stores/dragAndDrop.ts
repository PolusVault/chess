import { create } from "zustand";

type DndState = {
    isDragging: boolean;
    hoveredSquare?: string;
    draggingEl?: HTMLElement;
    draggingElRect?: DOMRect;
};

type DnDActions = {
    setHoveredSquare(sq: string): void;
    setDraggingEl(draggingEl: HTMLElement): void;
    stopDragging(): void;
};

const initialState: DndState = {
    isDragging: false,
};

export const useDnD = create<DndState & DnDActions>((set) => ({
    ...initialState,

    setHoveredSquare: (sq: string) =>
        set({
            hoveredSquare: sq,
        }),

    setDraggingEl: (draggingEl: HTMLElement) =>
        set({
            isDragging: true,
            draggingEl,
            draggingElRect: draggingEl.getBoundingClientRect(),
        }),

    stopDragging: () => set(initialState),
}));
