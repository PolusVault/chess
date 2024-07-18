import * as wasm from "friendly-chess";
import { memory } from "friendly-chess/friendly_chess_wasm_bg.wasm";

let chess = wasm.ChessWasm.new();


chess.load_fen("8/5P2/k7/8/8/8/8/7K w - - 0 1");
let a = new Uint8Array(memory.buffer, chess.board(), 256);

let board: {
  piece: number;
  color: "b" | "w";
}[] = [];

let c = 0;
for (let i = 0; i < a.length; i += 2) {
  if ((c & 0x88) !== 0) {
    c++;
    continue;
  }

  let color = a[i];
  let piece = a[i + 1];

  if (piece == 0) {
    board.push(undefined);
  } else {
    board.push({
      piece,
      color: color == 128 ? "b" : "w",
    });
  }

  c++;
}

let moves = chess.moves_for_square("f7");

console.log(moves);

// console.log(chess.get_fen());
// console.log(chess.turn());
