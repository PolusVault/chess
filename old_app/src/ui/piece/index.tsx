import pieces from "../../assets/pieces.svg";

type PieceProps = {
  type: string;
  color: string;
  square: string;
  flipped: boolean;
  onStartDragging: (e: React.MouseEvent, from: string, color: string) => void;
};

function Piece({ type, color, square, flipped, onStartDragging }: PieceProps) {
  return (
    <svg
      onMouseDown={(e) => onStartDragging(e, square, color)}
      // onTouchStart={onTouchStart}
      viewBox="0 0 45 45"
      className={`svg-piece ${flipped ? "flipped" : ""} sq-${square}`}
    >
      <use href={`${pieces}#piece-${color}-${type}`}></use>
    </svg>
  );
}

export default Piece;
