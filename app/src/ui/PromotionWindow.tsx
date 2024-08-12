import { useEffect, useRef } from "react";
import { PromotionPiece } from "./PromotionPiece";

type PromotionWindowProps = {
    isAwaitingPromotion: boolean;
    square: string;
    color: string;
    flipped: boolean;
    closePromotionWindow: () => void;
    onSelectPromotionPiece: (color: string, type: string) => void;
};

export default function PromotionWindow({
    isAwaitingPromotion,
    square,
    color,
    flipped,
    closePromotionWindow,
    onSelectPromotionPiece,
}: PromotionWindowProps) {
    const promotionWindowRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const onMouseDown = (e: MouseEvent) => {
            if (isAwaitingPromotion) {
                // if we click outside the promotion window, close it
                if (
                    promotionWindowRef.current &&
                    !promotionWindowRef.current.contains(e.target as Node)
                ) {
                    closePromotionWindow();
                }
            }
        };

        document.addEventListener("mousedown", onMouseDown);
        return () => {
            document.removeEventListener("mousedown", onMouseDown);
        };
    }, [isAwaitingPromotion]);

    useEffect(() => {
        if (isAwaitingPromotion) {
            const parts = square.split("");
            const file = parts[3].charCodeAt(0) - 97;
            const rank = parts[4];

            if (!flipped) {
                const yShift = parseInt(rank) === 1 ? "100%" : "0%";
                promotionWindowRef.current!.style.transform = `translate(${
                    file * 100
                }%, ${yShift})`;
            } else {
                const yShift = parseInt(rank) === 8 ? "100%" : "0%";
                promotionWindowRef.current!.style.transform = `translate(${
                    (7 - file) * 100
                }%, ${yShift})`;
            }
        }
    }, [isAwaitingPromotion, square]);

    return (
        <div
            ref={promotionWindowRef}
            className={`promotion-window ${isAwaitingPromotion ? square : ""}`}
            style={{
                display: isAwaitingPromotion ? "block" : "none",
            }}
        >
            <div className="promotion-piece-container">
                {["queen", "rook", "bishop", "knight"].map((type) => (
                    <PromotionPiece
                        key={type}
                        color={color}
                        type={type}
                        onSelectPromotionPiece={onSelectPromotionPiece}
                    />
                ))}
            </div>
        </div>
    );
}
