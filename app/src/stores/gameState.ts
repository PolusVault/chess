import { create } from "zustand";
import { Color } from "../chess";
import { PlayerInfo } from "../socket";

type GameState = {
    name: string;
    myColor: Color;
    room_id?: string;
    joinCode?: string;
    opponent?: PlayerInfo;
};

export type GameStateActions = {
    setName(name: GameState["name"]): void;
    setColor(color: GameState["myColor"]): void;
    setJoinCode(code: string): void;
    setOpponent(opponent: GameState["opponent"]): void;
    setRoomId(room_id: GameState["room_id"]): void;
};

const initialState: GameState = {
    name: "anonymous",
    myColor: "w",
};

export const useGameState = create<GameState & GameStateActions>((set) => ({
    ...initialState,

    setName: (name) => set({ name }),
    setColor: (myColor) => set({ myColor }),
    setJoinCode: (joinCode) => set({ joinCode }),
    setOpponent: (opponent) => set({ opponent }),
    setRoomId: (room_id) => set({ room_id }),
}));
