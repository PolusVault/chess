import { useEffect, useState } from "react";
import { Opponent, useChessStore } from "../../stores/chess";
import { WebsocketClient } from "../../websocket/type";
import { WebsocketChessClient } from "../../websocket/client";
import Board from "./board";
import { Color, Move } from "../../types";
import { DrawCondition, WinLoseCondition } from "../../chess_engine";
import Modal from "../../ui/modal";
import pieces from "../../assets/pieces.svg";
import toast, { Toaster } from "react-hot-toast";

function Chess() {
  const chessStore = useChessStore((state) => state);
  const [wsClient, setWsClient] = useState<WebsocketClient>(null);
  const [joinGameId, setJoinGameId] = useState("");
  const [isCreateGame, setIsCreateGame] = useState(false);
  const [isJoinGame, setIsJoinGame] = useState(false);
  const [modalIsOpen, setModalIsOpen] = useState(false);
  const [, rerender] = useState({});

  const connectToServer = (): WebsocketClient => {
    let wsClient = new WebsocketChessClient();

    return wsClient;
  };

  const createGame = () => {
    // TODO: only allows creating game if there isn't already a game
    connectToServer().onConnectionEstablished((wsClient) => {
      if (chessStore.myName === "") {
        chessStore.setMyName("Anonymous");
        wsClient.updateName("Anonymous");
      } else {
        wsClient.updateName(chessStore.myName);
      }

      wsClient.createGame(chessStore.gameName, chessStore.myColor);
      setWsClient(wsClient);
    });
    // notify();
  };

  const joinGame = () => {
    connectToServer().onConnectionEstablished((wsClient) => {
      if (chessStore.myName === "") {
        chessStore.setMyName("Anonymous");
        wsClient.updateName("Anonymous");
      } else {
        wsClient.updateName(chessStore.myName);
      }

      wsClient.joinGame(joinGameId);
      chessStore.setGameId(joinGameId);
      setWsClient(wsClient);
    });
  };

  const cancelGame = () => {
    if (!wsClient) return;

    wsClient.close();
    chessStore.resetGame();
  };

  const play_move = (m: Move) => {
    if (m.from == "" || m.to == null) return;
    if (m.to == m.from) return;

    try {
      chessStore.engine.move(m);

      if (wsClient) {
        wsClient.makeMove(m);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const switchColor = (color: Color) => {
    chessStore.setMyColor(color);
  };

  const renderModalContent = () => {
    if (isCreateGame) {
      return (
        <div>
          <p>new game</p>
          <div>
            <input
              value={gameName}
              onChange={(e) => setGameName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && createGame()}
              type="text"
              placeholder="game name"
              autoFocus
              className="w-full my-2 p-2 border rounded"
              disabled={!!gameId}
            />
          </div>
          <div className="w-full flex">
            <button
              className="w-1/2 flex justify-center border pointer-events-auto"
              onClick={() => switchColor("white")}
              disabled={!!gameId}
            >
              <svg viewBox="0 0 45 45" className={`color-select`}>
                <use href={`${pieces}#piece-white-king`}></use>
              </svg>
            </button>
            <button
              className="w-1/2 flex justify-center border pointer-events-auto"
              onClick={() => switchColor("black")}
              disabled={!!gameId}
            >
              <svg viewBox="0 0 45 45" className={`color-select`}>
                <use href={`${pieces}#piece-black-king`}></use>
              </svg>
            </button>
          </div>
          <button
            className="w-full mt-3 p-1 bg-[#B88761] text-white rounded disabled:opacity-50"
            onClick={createGame}
            disabled={!!gameId}
          >
            create
          </button>
        </div>
      );
    }

    if (isJoinGame) {
      return (
        <div>
          <p>join game</p>
          <div>
            <input
              value={joinGameId}
              onChange={(e) => setJoinGameId(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && joinGame()}
              type="text"
              placeholder="game code or link"
              autoFocus
              className="w-full my-2 p-2 border rounded"
            />
          </div>
          <button
            className="w-full mt-3 p-1 bg-[#B88761] text-white rounded"
            onClick={joinGame}
          >
            join
          </button>
        </div>
      );
    }

    if (gameOver) {
      return (
        <div>
          {gameStatus.win && <p>You won! 🎊</p>}
          {gameStatus.lose && <p>You lost 🙁</p>}
          {gameStatus.draw && <p>It's a draw!</p>}
          <p className="mt-3">By {gameOverReason}</p>
        </div>
      );
    }
  };

  // setup game subscribers
  useEffect(() => {
    const { engine } = chessStore;

    let unsubscribeToGameEngine = engine.subscribeToGameEvents((evt) => {
      switch (evt) {
        case "new-game":
          rerender({});
          break;
        case "move":
          rerender({});
          break;
        case "new-status":
          const myStatus = engine.gameStatus[chessStore.myColor];

          if (myStatus.win) {
            console.log(`You won by ${myStatus.win}`);
          } else if (myStatus.lose) {
            console.log(`You lose by ${myStatus.lose}`);
          } else if (engine.gameStatus.draw) {
            console.log(`Draw by ${engine.gameStatus.draw}`);
          }

          const newStatus = {
            ...myStatus,
            draw: engine.gameStatus.draw,
          };

          chessStore.setGameStatus(newStatus);
          setModalIsOpen(true);

          if (wsClient) {
            wsClient.updateGameState(engine.gameStatus);
          }

          break;
        default:
          break;
      }
    });

    return () => {
      unsubscribeToGameEngine();
    };
  }, [chessStore, wsClient]);

  // setup websockets
  useEffect(() => {
    if (!wsClient) return;

    let unsubscribeToWs = wsClient.onmessage((msg) => {
      switch (msg.type) {
        case "connect":
          console.log("Connected to WS server!");
          console.log(`My ID: ${msg.payload.id}`);
          chessStore.setMyId(msg.payload.id);
          break;
        case "creategame":
          console.log(msg.payload);
          chessStore.setGameId(msg.payload.id);
          toast.custom(
            (t) => (
              // TODO: why does it take forever to leave
              <ToastMessage visible={t.visible} gameId={msg.payload.id} />
            ),
            {
              id: "test",
              duration: Infinity,
            }
          );
          break;
        case "opponentjoined":
          console.log(`Opponent:`);
          console.log(msg.payload);
          let opponent = msg.payload;
          if (opponent.color === "w") {
            chessStore.setMyColor("black");
          } else {
            chessStore.setMyColor("white");
          }
          chessStore.setOpponent(msg.payload);
          console.log("dimiss toast");
          toast.remove();
          setModalIsOpen(false);
          setIsCreateGame(false);
          setIsJoinGame(false);
          break;
        case "makemove":
          console.log("move", msg);
          engine.move(msg.payload as Move);
          break;
        default:
          console.log(msg);
          break;
      }
    });

    return () => {
      unsubscribeToWs();
    };
  }, [wsClient]);

  useEffect(() => {
    engine.newGame();
  }, []);

  const {
    engine,
    myColor,
    myName,
    opponent,
    gameName,
    gameId,
    gameStatus,
    setGameName,
    setMyName,
  } = chessStore;

  const isWaitingForOpponent = gameId && !opponent;
  const isPlaying = gameId && opponent;

  const gameOver = gameStatus.win || gameStatus.lose || gameStatus.draw;
  let gameOverReason: WinLoseCondition | DrawCondition | null = null;
  if (gameOver) {
    if (gameStatus.win) gameOverReason = gameStatus.win;
    if (gameStatus.lose) gameOverReason = gameStatus.lose;
    if (gameStatus.draw) gameOverReason = gameStatus.draw;
  }

  return (
    <div className="chess-container">
      <OpponentInfo
        opponent={opponent}
        isWaitingForOpponent={isWaitingForOpponent}
      />
      <Board
        board={engine.board}
        play_move={play_move}
        myColor={myColor}
        getLegalMoves={(from: string) => engine.getLegalMoves(from)}
      />
      <input
        onFocus={(e) => e.target.select()}
        onBlur={(e) => setMyName(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            setMyName((e.target as HTMLInputElement).value);
          }
        }}
        onChange={(e) => setMyName(e.target.value)}
        value={
          myName === "" ? "Give yourself a name by clicking here!" : myName
        }
        className="my-name name w-full"
        type="text"
      />

      <section className="left-section">
        {!isPlaying || gameOver ? (
          <>
            <div>
              <button
                onClick={() => {
                  setIsCreateGame(true);
                  setIsJoinGame(false);
                  setModalIsOpen(true);
                }}
                className="text-2xl w-64  p-3 bg-gray-100 mx-auto block rounded text-center disabled:opacity-50 ease-in duration-100 hover:scale-[1.05]"
                disabled={!!gameId && !gameOver}
              >
                create a game
              </button>
            </div>
            <div>
              <button
                onClick={() => {
                  setIsJoinGame(true);
                  setIsCreateGame(false);
                  setModalIsOpen(true);
                }}
                disabled={!!gameId && !gameOver}
                className="text-2xl w-64 my-3 p-3 bg-gray-100 mx-auto block rounded text-center disabled:opacity-50 ease-in duration-100 hover:scale-[1.05]"
              >
                join a game
              </button>
            </div>
            <div>
              {isWaitingForOpponent && (
                <button
                  onClick={cancelGame}
                  className="text-2xl w-64 p-3 bg-gray-100 mx-auto block rounded text-center disabled:opacity-50 text-red-500 ease-in duration-100 hover:scale-[1.05]"
                >
                  cancel game
                </button>
              )}
            </div>
          </>
        ) : null}
      </section>

      <div>
        <Modal
          isOpen={modalIsOpen}
          onRequestClose={() => {
            toast.remove();
            setModalIsOpen(false);
          }}
        >
          {renderModalContent()}
        </Modal>
        <div>
          <Toaster
            position="bottom-center"
            containerStyle={{
              transform: "translate(0, -27%)",
              // marginTop: "2rem",
            }}
          />
        </div>
      </div>
    </div>
  );
}

type OpponentNameProps = {
  opponent: Opponent;
  isWaitingForOpponent: boolean;
};

function OpponentInfo({ opponent, isWaitingForOpponent }: OpponentNameProps) {
  if (isWaitingForOpponent) {
    return <p className="opponent-name name">Waiting for opponent...</p>;
  } else {
    return (
      <p className="opponent-name name">
        {opponent
          ? opponent.name
          : "create or join a game to play against someone"}
      </p>
    );
  }
}

type ToastMessageProps = {
  visible: boolean;
  gameId: string;
};

function ToastMessage({ visible, gameId }: ToastMessageProps) {
  const [isCopied, setIsCopied] = useState(false);

  return (
    <div
      className={`${
        visible ? "animate-enter" : "animate-leave"
      } max-w-[270px] w-full bg-white shadow-lg rounded-lg pointer-events-auto flex ring-1 ring-black ring-opacity-5`}
    >
      <div className="flex-1 w-0 p-4">
        <div className="flex items-start">
          <div className="ml-3 flex-1">
            <p className="text-sm font-medium text-gray-900">
              Game created! 🎉
            </p>
            <p className="mt-1 text-sm text-gray-500">Code: {gameId}</p>
          </div>
        </div>
      </div>
      <div className="flex border-l border-gray-200">
        <button
          onClick={() => {
            navigator.clipboard.writeText(gameId);
            setIsCopied(true);

            setTimeout(() => {
              setIsCopied(false);
            }, 2000);
          }}
          className="w-full border border-transparent rounded-none rounded-r-lg p-4 flex items-center justify-center text-sm font-medium text-[#B88761] focus:outline-none focus:ring-2 focus:ring-[#B88761]"
        >
          {isCopied ? "copied" : "copy"}
        </button>
      </div>
    </div>
  );
}

export default Chess;
