// import {create} from 'zustand';

import { create } from "zustand";

interface DndState {
    hoveredSquare: string | null;
    isDragging: boolean;
    draggingEl: HTMLElement | null;
    draggingElRect: DOMRect | null;
    setHoveredSquare: (sq: string) => void;
    setDraggingEl: (draggingEl: HTMLElement) => void; 
    stopDragging: () => void;
}

export const useDndStore = create<DndState>((set) => ({
    hoveredSquare: null,
    isDragging: false,
    draggingEl: null,
    draggingElRect: null,

    setHoveredSquare: (sq: string) => set({
        hoveredSquare: sq
    }),

    setDraggingEl: (draggingEl: HTMLElement) => set({
        isDragging: true,
        draggingEl,
        draggingElRect: draggingEl.getBoundingClientRect()
    }),

    stopDragging: () => set({
        draggingEl: null,
        draggingElRect: null,
        isDragging: false
    })
}));



