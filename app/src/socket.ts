import { io, Socket } from "socket.io-client";
import { Color, Move } from "./chess";

export type PlayerInfo = {
    name: string;
    color: Color;
};

type Payload<T> = {
    payload: T;
};
type AckPayload<T> = Payload<T> & {
    success: boolean;
};
type CreateGamePayload = { color: string; name: string };
type JoinGamePayload = { room_id: string; name: string };
type LeaveGamePayload = { room_id: string };
type MakeMovePayload = { move: Move; room_id: string };

type ClientToServerEvents = {
    "create-game": (
        payload: Payload<CreateGamePayload>,
        ack: (data: AckPayload<string>) => void
    ) => void;
    "join-game": (
        payload: Payload<JoinGamePayload>,
        ack: (data: AckPayload<PlayerInfo>) => void
    ) => void;
    "leave-game": (payload: Payload<LeaveGamePayload>, ack: () => void) => void;
    "make-move": (payload: Payload<MakeMovePayload>) => void;
};

type ServerToClientEvents = {
    "opponent-connected": (opponent: PlayerInfo) => void;
    "opponent-disconnected": (opponent: PlayerInfo) => void;
    "make-move": (move: Move) => void;
};

// "undefined" means the URL will be computed from the `window.location` object
const URL =
    import.meta.env.MODE === "production" ? undefined : "http://localhost:5000";

export const socket: Socket<ServerToClientEvents, ClientToServerEvents> = io(
    // @ts-ignore
    URL,
    {
        autoConnect: false,
        path: "/chess/socket",
    }
);

export function makePayload(data: any): Payload<any> {
    return {
        payload: data,
    };
}

interface EventsMap {
    [event: string]: any;
}

type EventNames<Map extends EventsMap> = keyof Map & (string | symbol);
type EventParams<
    Map extends EventsMap,
    Ev extends EventNames<Map>
> = Parameters<Map[Ev]>;

let emitEventsQueue: {
    ev: EventNames<ClientToServerEvents>;
    args: any;
}[] = [];

// once we are conneected, process all the queued events
socket.on("connect", () => {
    let event = emitEventsQueue.pop();

    while (event != undefined) {
        socket.emit(event.ev, ...event.args);
        event = emitEventsQueue.pop();
    }
});

export function emit<Ev extends EventNames<ClientToServerEvents>>(
    ev: Ev,
    ...args: EventParams<ClientToServerEvents, Ev>
) {
    if (socket.connected) {
        socket.emit(ev, ...args);
    } else {
        socket.connect();
        emitEventsQueue.push({ ev, args });
    }
}
