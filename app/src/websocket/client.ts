import { GameStatus } from "../chess_engine";
import { Color, Move } from "../types";
import { ServerMessage, ServerMessageType, WebsocketClient } from "./type";

function createServerMessage(type: ServerMessageType, payload: any): string {
  if ((typeof payload !== "object" && !Array.isArray(payload)) || !payload) {
    throw new Error("payload must be a valid object");
  }

  let msg: ServerMessage = {
    type,
    payload,
  };

  return JSON.stringify(msg);
}

export class WebsocketChessClient implements WebsocketClient {
  private socket: WebSocket | null;
  private subscribers: ((msg: ServerMessage) => void)[] = [];
  private onConnectionSubscribers: ((wsClient: WebsocketClient) => void)[] = [];
  private messagesQueue = [];

  constructor() {
    // let url = `wss://${window.location.hostname}/ws`;
    let url = "ws://127.0.0.1:8080/ws";

    if (!this.socket) {
      this.socket = new WebSocket(url);
      this.listen();
    }
  }

  createGame(name: string, color: Color) {
    if (!this.socket) {
      return;
    }

    if (name === "") return;

    this.socket.send(
      createServerMessage("creategame", {
        name,
        // we only need the first letter of the color
        color: color[0],
      })
    );
  }

  joinGame(game_id: string) {
    if (!this.socket) {
      return;
    }

    if (game_id === "") return;

    this.socket.send(
      createServerMessage("joingame", {
        game_id,
      })
    );
  }

  makeMove(m: Move): void {
    if (!this.socket) {
      return;
    }

    this.socket.send(createServerMessage("makemove", m));
  }

  updateGameState(s: GameStatus): void {
    if (!this.socket) {
      return;
    }

    // this.socket.send(
    //   createServerMessage("updatestatus", {
    //     new_status: s,
    //   })
    // );

    // Close the connection since the game ended
    // TODO: Figure out a better way to handle this
    // this.socket.close();
    queueMicrotask(() => {
      this.close();
    });
  }

  updateName(name: string) {
    if (!this.socket) {
      return;
    }

    if (name === "") return;

    this.socket.send(
      createServerMessage("updatename", {
        name,
      })
    );
  }

  onConnectionEstablished(cb: (wsClient: WebsocketClient) => void): () => void {
    this.onConnectionSubscribers.push(cb);

    return () => {
      let idx = this.onConnectionSubscribers.indexOf(cb);
      this.subscribers.splice(idx, 1);
    };
  }

  onmessage(cb: (msg: ServerMessage) => void): () => void {
    this.subscribers.push(cb);

    if (this.messagesQueue.length > 0) {
      this.subscribers.forEach((sub) =>
        this.messagesQueue.forEach((m) => sub(m))
      );
      this.messagesQueue = [];
    }

    return () => {
      let idx = this.subscribers.indexOf(cb);
      this.subscribers.splice(idx, 1);
    };
  }

  close() {
    this.socket.close();
    // this.onConnectionSubscribers = [];
    // this.subscribers = [];
  }

  private listen() {
    this.socket.onopen = () => {
      this.onConnectionSubscribers.forEach((sub) => sub(this));
    };

    this.socket.onclose = () => {
      this.socket = null;
    };

    this.socket.onmessage = (event) => {
      let msg = JSON.parse(event.data) as ServerMessage;

      // The messages queue ensures that every message is stored and processed
      // later if a wsClient.onmessage is initialized late, assuming
      // that we haven't already registered any subscribers
      this.messagesQueue.push(msg);

      this.subscribers.forEach((sub) => {
        const last = this.messagesQueue.length - 1;
        sub(this.messagesQueue[last]);
        this.messagesQueue.splice(last, 1);
      });
    };
  }
}
