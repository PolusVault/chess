import { useState, useEffect } from "react";
import toast, { Toaster } from "react-hot-toast";
import classNames from "classnames";
import Board from "../ui/Board";
import { useGameState } from "../stores/gameState";
import {
    chess,
    Move,
    onGameStatusChange,
    GameState,
    playMove,
    Color,
} from "../chess";
// import Button from "../ui/Button";
import PlayerInfo from "../ui/PlayerInfo";
import Modal from "../ui/Modal";

type Props = {
    leaveGame(): void;
    makeMove(move: Move): void;
    onOpponentMove(cb: () => void): () => void;
};

export default function Game({ onOpponentMove, makeMove, leaveGame }: Props) {
    const [sideToMove, setSideToMove] = useState(chess.turn());
    const [gameStatus, setGameStatus] = useState<{
        isOver: boolean;
        state: GameState;
        win?: boolean;
        lose?: boolean;
        draw?: boolean;
    } | null>(null);
    const [modal, setModal] = useState(false); // this is the modal to show the game status
    const { name, opponent, myColor, room_id } = useGameState((state) => ({
        ...state,
    }));

    useEffect(() => {
        function handleOpponentMove() {
            setSideToMove(chess.turn());
        }

        function handleGameStatus(state: GameState, color: Color) {
            if (state === GameState.Checkmate) {
                let status: { win?: boolean; lose?: boolean } = {};

                if (myColor == color) {
                    console.log("lost by " + GameState.Checkmate);
                    status.lose = true;
                } else {
                    console.log("won by " + GameState.Checkmate);
                    status.win = true;
                }

                setGameStatus({
                    isOver: true,
                    state: GameState.Checkmate,
                    ...status,
                });
                setModal(true);
            }
            if (
                state === GameState.Stalemate ||
                state === GameState.InsufficientMaterials ||
                state === GameState.ThreefoldRepetition
            ) {
                console.log("draw by " + state);
                setGameStatus({
                    isOver: true,
                    state,
                    draw: true,
                });
                setModal(true);
            }
        }

        let unsubOpponentMove = onOpponentMove(handleOpponentMove);
        let unsubGameStatusChange = onGameStatusChange(handleGameStatus);

        return () => {
            // unsubscribe
            unsubOpponentMove();
            unsubGameStatusChange();
        };
    }, []);

    useEffect(() => {
        if (!opponent) {
            return;
        }
        toast(`${opponent.name} has joined`, {
            icon: "🥊",
            position: "top-right",
            id: "opponent-joined", // to prevent duplicate toats
        });
    }, [opponent]);

    const _playMove = (m: Move) => {
        playMove(m);
        makeMove(m);
        setSideToMove(chess.turn());
    };

    const getLegalMoves = (sq: string): Move[] => {
        try {
            return chess.moves_for_square(sq);
        } catch {
            console.log("cant generate legal moves");
            return [];
        }
    };

    const renderGameStatus = () => {
        if (!gameStatus) return null;

        let reason;
        if (gameStatus.win) {
            reason = <p>You won by {gameStatus.state} 🥳</p>;
        }

        if (gameStatus.lose) {
            reason = <p>You lost by {gameStatus.state} 🙁</p>;
        }

        if (gameStatus.draw) {
            reason = <p>Game draw by {gameStatus.state}</p>;
        }

        return (
            <div>
                {reason}
                <button
                    onClick={leaveGame}
                    className="w-full rounded py-3 mt-5 bg-slate-100 text-lg"
                >
                    back to menu
                </button>
            </div>
        );
    };

    const waitingCls = classNames({ waiting: !opponent });

    return (
        <div className={`game-container ${waitingCls}`}>
            {opponent == undefined && (
                <div className="game-info text-lg">
                    <p>Invite code:</p>
                    <CopyMe room_id={room_id || ""} />
                    <p>The first to join will play against you.</p>
                    <button
                        onClick={leaveGame}
                        className="w-full rounded py-3 mt-5 bg-slate-50 text-red-500 text-lg"
                    >
                        cancel
                    </button>
                </div>
            )}
            <div className="opponent-info">
                <PlayerInfo playerInfo={opponent} />
            </div>
            <Board
                isPlaying={true}
                isOver={!!gameStatus}
                myColor={myColor}
                sideToMove={sideToMove}
                playMove={_playMove}
                flipped={myColor == "b"}
                getLegalMoves={getLegalMoves}
            />
            <div className="my-info">
                <PlayerInfo playerInfo={{ name, color: myColor }} />
            </div>

            <Modal
                isOpen={modal}
                onRequestClose={() => {
                    setModal(false);
                }}
            >
                <div>{renderGameStatus()}</div>
            </Modal>

            <Toaster />
        </div>
    );
}

type CopyMeProps = {
    room_id: string;
};

function CopyMe({ room_id }: CopyMeProps) {
    const [isCopied, setIsCopied] = useState(false);

    const copyCode = () => {
        navigator.clipboard.writeText(room_id);
        setIsCopied(true);
    };

    return (
        <div className="flex flex-nowrap my-1">
            <input
                className="w-full p-2 outline-none border rounded-l-lg placeholder-gray-500"
                value={room_id}
                readOnly
            />
            <button
                onClick={copyCode}
                className="text-base min-w-16 text-center justify-center flex-shrink-0 inline-flex items-center px-4 font-medium bg-slate-100 rounded-e-lg"
            >
                {isCopied ? <span>👍</span> : <span>copy</span>}
            </button>
        </div>
    );
}
