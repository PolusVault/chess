import { useState } from "react";
import Board from "../ui/Board";
import { chess, Move } from "../chess";
import { useGameState, GameStateActions } from "../stores/gameState";
import Button from "../ui/Button";
import Modal from "../ui/Modal";
import pieces from "../assets/pieces.svg";

type Props = {
    createGame(): void;
    joinGame(code: string): void;
};

export default function Home({ createGame, joinGame }: Props) {
    const [_myColor] = useState("w");
    const [sideToMove, setSideToMove] = useState(chess.turn());
    const [isCreatingGame, setIsCreatingGame] = useState(false);
    const [isJoiningGame, setIsJoiningGame] = useState(false);
    const { name, myColor, setName, setColor, setJoinCode, joinCode } =
        useGameState((state) => ({
            ...state,
        }));

    const playMove = (m: Move) => {
        chess.play_move(m);
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

    return (
        <div className="home-container">
            <section className="menu">
                <Button onClick={() => setIsCreatingGame(true)}>
                    create a game
                </Button>
                <Button onClick={() => setIsJoiningGame(true)} className="my-3">
                    join a game
                </Button>
            </section>

            <Board
                isPlaying={false}
                isOver={false}
                myColor={_myColor}
                sideToMove={sideToMove}
                playMove={playMove}
                flipped={myColor == "b"}
                getLegalMoves={getLegalMoves}
            />

            <Modal
                isOpen={isCreatingGame || isJoiningGame}
                onRequestClose={() => {
                    setIsCreatingGame(false);
                    setIsJoiningGame(false);
                }}
            >
                {isCreatingGame && (
                    <CreateGameModalContent
                        name={name}
                        setName={setName}
                        setColor={setColor}
                        createGame={createGame}
                    />
                )}

                {isJoiningGame && (
                    <JoinGameModal
                        joinCode={joinCode || ""}
                        setJoinCode={setJoinCode}
                        name={name}
                        setName={setName}
                        joinGame={joinGame}
                    />
                )}
            </Modal>
        </div>
    );
}

type CreateGameModalProps = {
    name: string;
    setName: GameStateActions["setName"];
    setColor: GameStateActions["setColor"];
    createGame(): void;
};

function CreateGameModalContent({
    name,
    setName,
    setColor,
    createGame,
}: CreateGameModalProps) {
    return (
        <div className="min-w-64">
            <p>new game</p>
            <div>
                <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && createGame()}
                    type="text"
                    placeholder="enter a name"
                    autoFocus
                    className="w-full my-2 p-2 border rounded placeholder-gray-500 focus:placeholder-opacity-75"
                    maxLength={15}
                />
            </div>
            <p className="mt-3">pick your color: </p>
            <div className="w-full flex mt-2">
                <button
                    className="w-1/2 flex justify-center border pointer-events-auto"
                    onClick={() => setColor("w")}
                >
                    <svg viewBox="0 0 45 45" className={`color-select`}>
                        <use href={`${pieces}#piece-w-king`}></use>
                    </svg>
                </button>
                <button
                    className="w-1/2 flex justify-center border pointer-events-auto"
                    onClick={() => setColor("b")}
                >
                    <svg viewBox="0 0 45 45" className={`color-select`}>
                        <use href={`${pieces}#piece-b-king`}></use>
                    </svg>
                </button>
            </div>
            <button
                className="w-full mt-3 p-1 bg-[#B88761] text-white rounded disabled:opacity-50"
                onClick={createGame}
            >
                create
            </button>
        </div>
    );
}

type JoinGameModalProps = {
    name: string;
    joinCode: string;
    setName: GameStateActions["setName"];
    setJoinCode: GameStateActions["setJoinCode"];
    joinGame(code: string): void;
};

function JoinGameModal({
    joinCode,
    name,
    setName,
    setJoinCode,
    joinGame,
}: JoinGameModalProps) {
    return (
        <div className="min-w-64">
            <p>join game</p>
            <div className="my-2">
                <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    type="text"
                    placeholder="enter a name"
                    autoFocus
                    className="w-full my-2 p-2 border rounded placeholder-gray-500 focus:placeholder-opacity-75"
                    maxLength={15}
                />
            </div>
            <div>
                <input
                    value={joinCode}
                    onChange={(e) => setJoinCode(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && joinGame(joinCode)}
                    type="text"
                    placeholder="enter join code"
                    autoFocus
                    className="w-full mb-2 p-2 border rounded placeholder-gray-500 focus:placeholder-opacity-75"
                    maxLength={15}
                />
            </div>
            <button
                className="w-full mt-3 p-1 bg-[#B88761] text-white rounded disabled:opacity-50"
                onClick={() => joinGame(joinCode)}
            >
                join
            </button>
        </div>
    );
}
