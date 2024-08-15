import { useRef, useEffect, useState } from "react";
import classNames from "classnames";
import { getBoard, getSquareNotation } from "../chess";
import { useDnD } from "../stores/dragAndDrop";
import Piece from "./Piece";
import { Move } from "../chess";
import PromotionWindow from "./PromotionWindow";

type Props = {
    isPlaying: boolean;
    isOver: boolean;
    flipped: boolean;
    sideToMove: string;
    myColor: string;
    playMove(m: Move): void;
    getLegalMoves(sq: string): Move[];
};

function clamp(value: number, min: number, max: number) {
    return Math.max(Math.min(value, max), min);
}

// function invariant() {}

export default function Board({
    isPlaying,
    isOver,
    flipped,
    myColor,
    sideToMove,
    playMove,
    getLegalMoves,
}: Props) {
    const boardEl = useRef<HTMLDivElement>(null);
    const legalMovesRef = useRef<Move[]>([]);
    const moveRef = useRef<Move>({});
    const hoverSquareRef = useRef<HTMLDivElement>(null);

    const dnd = useDnD((state) => ({ ...state }));

    const [isAwaitingPromotion, setIsPromoting] = useState(false);

    const flippedCls = classNames({ flipped });

    useEffect(() => {
        // dragging
        const onMouseMove = (e: MouseEvent) => {
            if (!dnd.isDragging) return;
            if (!moveRef.current) return;

            const boardRect = boardEl.current!.getBoundingClientRect();
            const pieceRect = dnd.draggingElRect!;

            const x = clamp(e.clientX - boardRect.left, 0, boardRect.width);
            const y = clamp(e.clientY - boardRect.top, 0, boardRect.height);

            const pieceX = Math.floor(x - pieceRect.width / 2);
            const pieceY = Math.floor(y - pieceRect.height / 2);

            dnd.draggingEl!.style.transform = `translate(${pieceX}px, ${pieceY}px)`;

            const file = clamp(Math.floor(x / pieceRect.width), 0, 7);
            const rank = clamp(8 - Math.floor(y / pieceRect.height), 1, 8);
            const to = `${
                flipped
                    ? String.fromCharCode(201 - (97 + file))
                    : String.fromCharCode(97 + file)
            }${flipped ? 9 - rank : rank}`;

            moveRef.current!.to = to;

            hoverSquareRef.current!.className = `hover-square border-4 border-slate-300 sq-${to} ${flippedCls}`;
        };

        // on drop
        const onMouseUp = () => {
            if (!dnd.isDragging) return;
            if (!moveRef.current) return;
            if (!dnd.draggingEl) return;

            dnd.draggingEl.classList.toggle("dragging");
            dnd.draggingEl.removeAttribute("style");

            hoverSquareRef.current!.removeAttribute("style");
            hoverSquareRef.current!.className = "hover-square";

            // remove highlight elements
            let highlights = document.getElementsByClassName("highlight");
            Array.from(highlights).forEach((el: Element) => el.remove());

            const legalMoves = legalMovesRef.current;

            if (
                legalMoves.filter(
                    (m) =>
                        m.from == moveRef.current.from &&
                        m.to == moveRef.current.to
                ).length > 0
            ) {
                // if promoting, display the promotion window to select which piece to promote to
                if (legalMoves[0].promotion_piece) {
                    setIsPromoting(true);
                } else {
                    try {
                        playMove(moveRef.current);
                    } catch (e) {
                        console.log(moveRef.current);
                        console.log("ERROR", e);
                    }
                    moveRef.current = {};
                }
            }

            dnd.stopDragging();

            legalMovesRef.current = [];
        };

        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", onMouseUp);

        return () => {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
        };
    }, [dnd.isDragging, isAwaitingPromotion]);

    const onStartDragging = (
        e: React.MouseEvent,
        from: string,
        color: string
    ) => {
        e.preventDefault();

        if (!boardEl.current) return;
        if (!moveRef.current) return;
        if (isPlaying && myColor != color) return;
        if (isOver) return;

        moveRef.current = {
            from,
        };

        legalMovesRef.current = color == sideToMove ? getLegalMoves(from) : [];

        // highlight legal squares
        legalMovesRef.current.forEach((m) => {
            // create a div that highlights the square
            const highlightDiv = document.createElement("div");
            highlightDiv.className = `highlight ${flippedCls} sq-${m.to}`;

            const highlightChild = document.createElement("div");

            highlightChild.className = `block rounded-full bg-slate-200`;
            highlightDiv.appendChild(highlightChild);

            boardEl.current!.appendChild(highlightDiv);
        });

        const boardRect = boardEl.current.getBoundingClientRect();
        const pieceRect = e.currentTarget.getBoundingClientRect();

        const boardx = e.clientX - boardRect.left;
        const boardy = e.clientY - boardRect.top;

        let file = clamp(Math.floor(boardx / pieceRect.width), 0, 7);
        let rank = clamp(8 - Math.floor(boardy / pieceRect.height), 1, 8);

        hoverSquareRef.current!.style.visibility = "visible";
        hoverSquareRef.current!.classList.toggle(
            `sq-${String.fromCharCode(97 + file)}${rank}`
        );

        // snap the center of the piece to the cursor
        let newTranslateX = boardx - pieceRect.width / 2;
        let newTranslateY = boardy - pieceRect.height / 2;

        (
            e.currentTarget as HTMLElement
        ).style.transform = `translate(${newTranslateX}px, ${newTranslateY}px)`;
        e.currentTarget.classList.toggle("dragging");
        dnd.setDraggingEl(e.currentTarget as HTMLElement);
    };

    const onSelectPromotionPiece = (color: string, type: string) => {
        if (!moveRef.current) return;

        try {
            switch (type) {
                case "knight":
                    moveRef.current.promotion_piece = "n";
                    break;
                case "queen":
                case "rook":
                case "bishop":
                    if (color === "w") {
                        moveRef.current.promotion_piece = type[0].toUpperCase();
                    } else {
                        moveRef.current.promotion_piece = type[0];
                    }
            }

            playMove(moveRef.current);
            moveRef.current = {};
        } catch (e) {
            console.error(e);
        }

        closePromotionWindow();
    };

    const closePromotionWindow = () => {
        setIsPromoting(false);
    };

    const renderPieces = (): JSX.Element[] => {
        let pieces = [];

        for (const piece of getBoard()) {
            if (!piece) continue;

            let square = getSquareNotation(piece.rank, piece.file);
            pieces.push(
                <Piece
                    onStartDragging={onStartDragging}
                    piece={piece}
                    square={square}
                    key={square}
                    flipped={flipped}
                />
            );
        }

        return pieces;
    };

    return (
        <div className={`chess-board ${flippedCls}`} ref={boardEl}>
            <PromotionWindow
                isAwaitingPromotion={isAwaitingPromotion}
                square={`sq-${moveRef.current.to}`}
                closePromotionWindow={closePromotionWindow}
                onSelectPromotionPiece={onSelectPromotionPiece}
                color={sideToMove}
                flipped={flipped}
            />
            <div
                ref={hoverSquareRef}
                className="hover-square border-4 border-slate-300"
            ></div>
            {renderPieces()}
        </div>
    );
}
