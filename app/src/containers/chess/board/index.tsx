import BoardUI from "../../../ui/board";
import { Color, Move } from "../../../types";

interface BoardProps {
  board: Uint8Array;
  myColor: Color;
  play_move: (m: Move) => void;
  getLegalMoves: (sq: string) => Move[];
}

function Board(props: BoardProps) {
  return <BoardUI {...props} />;
}

export default Board;
