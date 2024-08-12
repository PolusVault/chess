import svgPieces from "../assets/pieces.svg";
import { Piece as TPiece } from "../chess";
import classNames from "classnames";

type Props = {
    piece: TPiece;
    square: string;
    flipped: boolean;
    onStartDragging: (e: React.MouseEvent, from: string, color: string) => void;
};

export default function Piece({
    flipped,
    onStartDragging,
    piece,
    square,
}: Props) {
    const flippedCls = classNames({ flipped });
    return (
        <svg
            onMouseDown={(e) => onStartDragging(e, square, piece.color)}
            viewBox="0 0 45 45"
            className={`svg-piece sq-${square} ${flippedCls}`}
            key={square}
        >
            <use href={`${svgPieces}#piece-${piece.color}-${piece.type}`}></use>
        </svg>
    );
}
