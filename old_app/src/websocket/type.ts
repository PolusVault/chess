import { GameStatus } from "../chess_engine";
import { Color, Move } from "../types";

export type ServerMessageType =
    | "connect"
    | "creategame"
    | "joingame"
    | "opponentjoined"
    | "makemove"
    | "updatestatus"
    | "updatename";

export interface ServerMessage {
    type: ServerMessageType;
    payload: any;
}

export interface WebsocketClient {
    createGame(name: string, color: Color): void;
    joinGame(game_id: string): void;
    makeMove(m: Move): void;
    updateGameState(s: GameStatus): void;
    updateName(name: string): void;
    close(): void;
    onmessage(cb: (msg: ServerMessage) => void): () => void;
    onConnectionEstablished(cb: (wsClient: WebsocketClient) => void): () => void;
}
