import { useState, useEffect, useRef } from "react";
import toast, { Toaster } from "react-hot-toast";
import Home from "./modules/Home";
import Game from "./modules/Game";
import { socket, PlayerInfo, makePayload, emit } from "./socket";
import { useGameState } from "./stores/gameState";
import { Move, playMove } from "./chess";
import "./App.css";

export default function App() {
    const [isPlaying, setIsPlaying] = useState(false);
    const [isConnected, setIsConnected] = useState(socket.connected);
    const roomInfoRef = useRef<{ room_id: string } | null>(null);
    const { myColor, name, setOpponent, setColor, setRoomId } = useGameState(
        (state) => ({
            ...state,
        })
    );
    const onOpponentMoveSubscribers = useRef<(() => void)[]>([]);

    useEffect(() => {
        function onConnect() {
            setIsConnected(true);
        }

        function onDisconnect() {
            setIsConnected(false);
        }

        function onOpponentConnect(opponent: PlayerInfo) {
            setOpponent(opponent);
        }

        function onOpponentDisconnect() {
            toast("🏃 your opponent disconnected", {
                position: "top-right",
            });
            setOpponent(undefined);
        }

        function onMove(move: Move) {
            try {
                playMove(move);
                onOpponentMoveSubscribers.current.forEach((cb) => cb());
            } catch (e) {
                console.log(e);
            }
        }

        socket.on("connect", onConnect);
        socket.on("disconnect", onDisconnect);
        socket.on("opponent-connected", onOpponentConnect);
        socket.on("opponent-disconnected", onOpponentDisconnect);
        socket.on("make-move", onMove);

        return () => {
            socket.off("connect", onConnect);
            socket.off("disconnect", onDisconnect);
            socket.off("opponent-connected", onOpponentConnect);
            socket.off("opponent-disconnected", onOpponentDisconnect);
            socket.off("make-move", onMove);
        };
    }, []);

    const createGame = () => {
        emit("create-game", makePayload({ color: myColor, name }), (res) => {
            if (!res.success) {
                toast.error("error when creating room");
                return;
            }
            roomInfoRef.current = { room_id: res.payload };
            setIsPlaying(true);
            setRoomId(roomInfoRef.current.room_id);
        });
    };

    const joinGame = (code: string) => {
        emit("join-game", makePayload({ room_id: code, name }), (res) => {
            if (!res.success) {
                toast.error("error when joining room");
                return;
            }
            roomInfoRef.current = {
                room_id: code,
            };
            setOpponent(res.payload);
            setColor(res.payload.color === "w" ? "b" : "w");
            setIsPlaying(true);
            setRoomId(code);
        });
    };

    const leaveGame = () => {
        if (!socket.connected) return;
        if (!roomInfoRef.current) return;

        emit(
            "leave-game",
            makePayload({ room_id: roomInfoRef.current.room_id }),
            () => {
                console.log("left the game");
                roomInfoRef.current = null;
                setIsPlaying(false);
            }
        );
    };

    const makeMove = (move: Move) => {
        emit(
            "make-move",
            makePayload({ move, room_id: roomInfoRef.current!.room_id })
        );
    };

    const onOpponentMove = (cb: () => void) => {
        onOpponentMoveSubscribers.current.push(cb);

        return () => {
            for (let i = 0; i < onOpponentMoveSubscribers.current.length; i++) {
                const sub = onOpponentMoveSubscribers.current[i];
                if (sub == cb) {
                    onOpponentMoveSubscribers.current.splice(i, 1);
                }
            }
        };
    };

    return (
        <div className="app-container">
            {isPlaying ? (
                <Game
                    onOpponentMove={onOpponentMove}
                    leaveGame={leaveGame}
                    makeMove={makeMove}
                />
            ) : (
                <Home createGame={createGame} joinGame={joinGame} />
            )}

            <Toaster />
        </div>
    );
}
