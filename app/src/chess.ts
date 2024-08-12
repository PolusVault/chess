import init, { ChessWasm } from "hieu-chess-wasm";
import wasmUrl from "hieu-chess-wasm/hieu_chess_wasm_bg.wasm?url";

export type Piece = {
    type: string;
    color: string;
    rank: number;
    file: number;
};

export type Move = {
    from?: string;
    to?: string;
    promotion_piece?: string;
};

export type Color = "w" | "b";

export enum Status {
    Win,
    Lose,
    Draw,
}

export enum GameState {
    InProgress,
    Draw = "draw",
    Stalemate = "statemate",
    Checkmate = "checkmate",
    InsufficientMaterials = "insufficient materials",
    ThreefoldRepetition = "threefold repetition",
}

async function initChessLib() {
    const instance = await init(wasmUrl);

    const chess = ChessWasm.new();
    const board = new Uint8Array(instance.memory.buffer, chess.board(), 256);

    chess.reset();
    chess.load_fen("r2k4/8/8/8/5P2/8/p7/2K5 w - - 0 1");

    return {
        chess,
        board,
    };
}

export const { chess, board: _board } = await initChessLib();

export function getBoard(): (Piece | null)[] {
    let board = [];

    for (let i = 0; i < _board.length; i += 2) {
        let idx = i / 2;

        if ((idx & 0x88) !== 0) {
            continue;
        }

        if (_board[i] == 0) {
            board.push(null);
            continue;
        }

        let piece = _board[i];
        let color = _board[i + 1];
        let type;

        switch (piece) {
            case 1:
                type = "pawn";
                break;
            case 2:
                type = "knight";
                break;
            case 4:
                type = "bishop";
                break;
            case 8:
                type = "rook";
                break;
            case 16:
                type = "queen";
                break;
            case 32:
                type = "king";
                break;
            default:
                throw new Error("invalid piece type");
        }

        board.push({
            type,
            color: color == 0 ? "w" : "b",
            rank: idx >> 4,
            file: idx & 7,
        });
    }

    return board;
}

type StatusCb = (state: GameState, color: Color) => void;
const statusSubscribers: StatusCb[] = [];
export function onGameStatusChange(cb: StatusCb) {
    statusSubscribers.push(cb);

    return () => {
        for (let i = 0; i < statusSubscribers.length; i++) {
            const sub = statusSubscribers[i];
            if (sub == cb) {
                statusSubscribers.splice(i, 1);
            }
        }
    };
}

export function playMove(m: Move) {
    chess.play_move(m);

    let state = GameState.InProgress;

    if (chess.is_checkmate()) {
        state = GameState.Checkmate;
    } else if (chess.is_stalemate()) {
        state = GameState.Stalemate;
    } else if (chess.is_insufficient_materials()) {
        state = GameState.InsufficientMaterials;
    } else if (chess.is_threefold_repetition()) {
        state = GameState.ThreefoldRepetition;
    }

    if (state != GameState.InProgress) {
        statusSubscribers.forEach((cb) => cb(state, chess.turn() as Color));
    }
}

export function getSquareNotation(rank: number, file: number): string {
    return `${String.fromCharCode(97 + file)}${rank + 1}`;
}
