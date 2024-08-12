import { useEffect, useLayoutEffect, useRef } from 'react';
import pieces from '../../assets/pieces.svg';

type PromotionWindowProps = {
    isAwaitingPromotion: boolean;
    square: string;
    color: string;
    closePromotionWindow: () => void;
    onSelectPromotionPiece: (color: string, type: string) => void;
}

export function PromotionWindow({
    isAwaitingPromotion,
    square,
    color,
    closePromotionWindow,
    onSelectPromotionPiece
}: PromotionWindowProps) {
    const promotionWindowRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const onMouseDown = (e: MouseEvent) => {
            if (isAwaitingPromotion) {
                // if we click outside the promotion window, close it
                if (promotionWindowRef.current && !promotionWindowRef.current.contains(e.target as Node)) {
                    closePromotionWindow();
                }
            }
        };

        document.addEventListener("mousedown", onMouseDown);
        return () => {
            document.removeEventListener("mousedown", onMouseDown);
        }
    }, [isAwaitingPromotion])

    useLayoutEffect(() => {
        if (isAwaitingPromotion) {
            const parts = square.split("");
            const fileChar = parts[3];
            const rank = parts[4];

            let file = fileChar.charCodeAt(0) - 97;

            if (parseInt(rank) === 1) {
                promotionWindowRef.current.style.transform = `translate(${file * 100}%, 100%)`;
            } else {
            }

        }
    }, [isAwaitingPromotion, square])

    return (
        <div ref={promotionWindowRef}
            className={`promotion-window ${isAwaitingPromotion ? square : ""}`}
            style={{
                display: isAwaitingPromotion ? "block" : "none"
            }}
        >
            <div className="promotion-piece-container">
                {["queen", "rook", "bishop", "knight"].map(type => (
                    <PromotionPiece key={type} color={color} type={type} onSelectPromotionPiece={onSelectPromotionPiece} />
                ))}
            </div>

        </div>

    )
}

type PromotionPieceProps = {
    color: string;
    type: string;
    onSelectPromotionPiece: (color: string, type: string) => void;
}

export function PromotionPiece({ color, type, onSelectPromotionPiece }: PromotionPieceProps) {
    return (
        <svg
            viewBox="0 0 45 45"
            className={`promotion-piece`}
            onClick={() => onSelectPromotionPiece(color, type)}
        >
            <use href={`${pieces}#piece-${color}-${type}`}></use>
        </svg>

    )

}





