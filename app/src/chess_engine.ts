import { ChessWasm } from "friendly-chess-wasm";
import { memory } from "friendly-chess-wasm/friendly_chess_wasm_bg.wasm";
import { Move } from "./types";

type GameEvent = "move" | "new-game" | "new-status";

export enum WinLoseCondition {
  Checkmate = "checkmate",
  Resign = "resign",
  Overtime = "overtime",
}

export enum DrawCondition {
  InsufficientMaterial = "insufficient_material",
  Stalemate = "stalemate",
  Repetition = "repetition",
}

export interface GameStatus {
  white: PlayerStatus;
  black: PlayerStatus;
  draw: DrawCondition | null;
}

export interface PlayerStatus {
  win: WinLoseCondition | null;
  lose: WinLoseCondition | null;
  // draw: DrawCondition | null;
}

export class ChessEngine {
  private wasmEngine: ChessWasm;

  private defaultFen =
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
  // private defaultFen = "k7/2Q5/1K6/8/8/8/8/8 w - - 0 1";
  // Subscribers will be notified whenever any events happen, like making a move
  private subscribers: ((eventType: GameEvent) => void)[] = [];
  public gameStatus: GameStatus = {
    draw: null,
    white: {
      win: null,
      lose: null,
    },
    black: {
      win: null,
      lose: null,
    },
  };

  public board: Uint8Array;
  public isGameInProgress: boolean = false;
  // public myColor: Color;
  // public opponent: Opponent | null;
  // public gameStatus: GameStatus;

  constructor() {
    console.log("initializing chess engine...");
    this.wasmEngine = ChessWasm.new();
    this.board = new Uint8Array(memory.buffer, this.wasmEngine.board(), 256);
    // this.myColor = "w";
  }

  public newGame(fen: string = this.defaultFen) {
    this.wasmEngine.load_fen(fen);
    this.notifySubscribers("new-game");
    this.isGameInProgress = true;
  }

  public move(m: Move) {
    this.wasmEngine.play_move(m);
    this.notifySubscribers("move");
    this.checkGameState();
  }

  public getLegalMoves(sq: string): Move[] {
    return this.wasmEngine.moves_for_square(sq);
  }

  public can_move({ from, to }: Move): boolean {
    const moves = this.wasmEngine.moves_for_square(from) as Move[];

    return moves.some((move) => move.from === from && move.to === to);
  }

  public subscribeToGameEvents(cb: (evt: GameEvent) => void): () => void {
    this.subscribers.push(cb);

    return () => {
      let idx = this.subscribers.indexOf(cb);
      this.subscribers.splice(idx, 1);
    };
  }

  private checkGameState() {
    let engine = this.wasmEngine;

    if (engine.is_checkmate()) {
      if (engine.turn() === "w") {
        this.gameStatus.white = {
          ...this.gameStatus.white,
          lose: WinLoseCondition.Checkmate,
        };
        this.gameStatus.black = {
          ...this.gameStatus.black,
          win: WinLoseCondition.Checkmate,
        };
      } else {
        this.gameStatus.black = {
          ...this.gameStatus.black,
          lose: WinLoseCondition.Checkmate,
        };
        this.gameStatus.white = {
          ...this.gameStatus.white,
          win: WinLoseCondition.Checkmate,
        };
      }

      this.notifySubscribers("new-status");
      this.isGameInProgress = false;
    } else if (engine.is_repetition()) {
      this.gameStatus.draw = DrawCondition.Repetition;
      this.notifySubscribers("new-status");
      this.isGameInProgress = false;
    } else if (engine.is_insufficient_materials()) {
      this.gameStatus.draw = DrawCondition.InsufficientMaterial;
      this.notifySubscribers("new-status");
      this.isGameInProgress = false;
    } else if (engine.is_stalemate()) {
      this.gameStatus.draw = DrawCondition.Stalemate;
      this.notifySubscribers("new-status");
      this.isGameInProgress = false;
    }
  }

  private notifySubscribers(eventType: GameEvent) {
    this.subscribers.forEach((sub) => sub(eventType));
  }
}
