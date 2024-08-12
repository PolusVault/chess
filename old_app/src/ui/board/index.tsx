import { useEffect, useRef, useState } from "react";
import { shallow } from "zustand/shallow";
import { useDndStore } from "../../stores/dnd";
import { clamp } from "../../utils";
import { Color, Move } from "../../types";
import { PromotionWindow } from "../promotion";
import Piece from "../piece";

type BoardProps = {
    board: Uint8Array;
    myColor: Color;
    play_move: (m: Move) => void;
    getLegalMoves: (sq: string) => Move[];
};

function Board({ board, myColor, play_move, getLegalMoves }: BoardProps) {
    const boardEl = useRef<HTMLDivElement | null>(null);
    const boardRectRef = useRef<DOMRect | null>(null);
    const hoverSquareRef = useRef<HTMLDivElement | null>(null);
    // TODO: handle this in the library
    const legalMovesRef = useRef<Move[]>([]);
    const moveRef = useRef<{
        color: string;
        from: string;
        to: string | null;
        promotion_piece: string | null;
    } | null>(null);
    const dnd = useDndStore((state) => ({ ...state }), shallow);
    const [isAwaitingPromotion, setIsPromoting] = useState(false);

    const flipped = myColor === "black";

    const onStartDragging = (
        e: React.MouseEvent,
        from: string,
        color: string
    ) => {
        e.preventDefault();
        if (myColor !== color) return;
        if (!boardEl.current) return;
        if (!hoverSquareRef.current) return;
        if (from == "") return;

        moveRef.current = {
            color,
            from,
            to: null,
            promotion_piece: null,
        };

        const legalMoves = getLegalMoves(from);

        legalMovesRef.current = legalMoves;
        // highlight legal squares
        legalMoves.forEach((m) => {
            // create a div that highlights the square
            const highlightDiv = document.createElement("div");
            highlightDiv.className = `highlight ${flipped ? "flipped" : ""} sq-${m.to
                }`;

            const highlightChild = document.createElement("div");

            highlightChild.className = `block rounded-full bg-slate-200`;
            highlightDiv.appendChild(highlightChild);

            boardEl.current.appendChild(highlightDiv);
        });

        const boardRect = boardRectRef.current;
        const pieceRect = e.currentTarget.getBoundingClientRect();

        const boardx = e.clientX - boardRect.left;
        const boardy = e.clientY - boardRect.top;

        let file = clamp(Math.floor(boardx / pieceRect.width), 0, 7);
        let rank = clamp(7 - Math.floor(boardy / pieceRect.height) + 1, 1, 8);

        hoverSquareRef.current.style.visibility = "visible";
        hoverSquareRef.current.classList.toggle(
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

    const onSelectPromotionPiece = (_: string, type: string) => {
        if (!moveRef.current) return;

        try {
            moveRef.current.promotion_piece = type;
            play_move(moveRef.current);
        } catch {
            console.error("illegal promotion");
        }

        closePromotionWindow();
    };

    const closePromotionWindow = () => {
        setIsPromoting(false);
    };

    const renderBoard = () => {
        const pieces: JSX.Element[] = [];
        let idx = 0;
        for (let i = 0; i < board.length; i += 2) {
            if ((idx & 0x88) !== 0) {
                idx++;
                continue;
            }

            const color = board[i];
            const piece = board[i + 1];

            // skip rendering empty square
            if (piece == 0) {
                idx++;
                continue;
            }

            const rank = 8 - ((idx >> 4) + 1) + 1;
            const file = idx & 7;

            let type = "";

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

            pieces.push(
                <Piece
                    square={`${String.fromCharCode(97 + file)}${rank}`}
                    color={color == 0 ? "white" : "black"}
                    type={type}
                    onStartDragging={onStartDragging}
                    key={idx}
                    flipped={flipped}
                />
            );

            idx++;
        }
        return pieces;
    };

    useEffect(() => {
        // dragging
        const onMouseMove = (e: MouseEvent) => {
            if (!dnd.isDragging) return;
            if (!boardRectRef.current) return;
            // moveRef must be initialized by onStartDragging, before this functions run
            if (!moveRef.current) return;

            const boardRect = boardRectRef.current;
            const pieceRect = dnd.draggingElRect;

            const x = clamp(e.clientX - boardRect.left, 0, boardRect.width);
            const y = clamp(e.clientY - boardRect.top, 0, boardRect.height);

            let file = clamp(Math.floor(x / pieceRect.width), 0, 7);
            let rank = clamp(7 - Math.floor(y / pieceRect.height) + 1, 1, 8);
            let to = `${flipped
                ? String.fromCharCode(201 - (97 + file))
                : String.fromCharCode(97 + file)
                }${flipped ? 9 - rank : rank}`;

            moveRef.current.to = to;

            hoverSquareRef.current.className = `hover-square border-4 border-slate-300 ${flipped ? "flipped" : ""
                } sq-${to}`;
            // hoverSquareRef.current.classList.toggle(`sq-${String.fromCharCode(97 + file)}${rank}`);

            let pieceX = Math.floor(x - pieceRect.width / 2);
            let pieceY = Math.floor(y - pieceRect.height / 2);

            dnd.draggingEl.style.transform = `translate(${pieceX}px, ${pieceY}px)`;
        };

        // on drop
        const onMouseUp = () => {
            if (!dnd.isDragging) return;
            if (!moveRef.current) return;

            dnd.draggingEl.classList.toggle("dragging");
            dnd.draggingEl.removeAttribute("style");

            hoverSquareRef.current.removeAttribute("style");
            hoverSquareRef.current.className = "hover-square";
            // dnd.draggingEl.style.removeProperty("transform");

            // remove highlight elements
            let highlights = document.getElementsByClassName("highlight");
            Array.from(highlights).forEach((el: Element) => el.remove());

            const legalMoves = legalMovesRef.current;

            if (
                legalMoves.filter(
                    (m) => m.from == moveRef.current.from && m.to == moveRef.current.to
                ).length > 0
            ) {
                // if promoting, display the promotion window to select which piece to promote to
                if (legalMoves[0].promotion_piece) {
                    // promotionWindowRef.current.style.display = "block";
                    // promotionWindowRef.current.classList.toggle(`sq-${moveRef.current.to}`);
                    setIsPromoting(true);
                } else {
                    play_move(moveRef.current);
                }
            }

            dnd.stopDragging();
            legalMovesRef.current = [];
        };

        // const onMouseDown = (e: MouseEvent) => {
        // }
        // document.addEventListener("mousedown", onMouseDown);
        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", onMouseUp);

        return () => {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
            // document.removeEventListener("mousedown", onMouseDown);
        };
    }, [dnd.isDragging, isAwaitingPromotion]);

    return (
        <div
            className={`chess-board ${flipped ? "flipped" : ""}`}
            ref={(el) => {
                if (!el) return;
                boardEl.current = el;
                boardRectRef.current = el.getBoundingClientRect();
            }}
            onMouseDown={(e) => e.preventDefault()}
        >
            <PromotionWindow
                isAwaitingPromotion={isAwaitingPromotion}
                square={
                    moveRef.current
                        ? `${flipped ? "flipped" : ""} sq-${moveRef.current.to}`
                        : ""
                }
                closePromotionWindow={closePromotionWindow}
                onSelectPromotionPiece={onSelectPromotionPiece}
                color={moveRef.current ? moveRef.current.color : undefined}
            />
            <div
                ref={hoverSquareRef}
                className="hover-square border-4 border-slate-300"
            ></div>
            {renderBoard()}
        </div>
    );
}

export default Board;
