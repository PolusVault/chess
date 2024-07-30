import shortuuid
from flask import request
from flask_socketio import rooms, leave_room, close_room


class Client:
    def __init__(self, id, name="Anonymous"):
        self.id = id
        self.name = name
        self.owned_rooms = []

    def get_info(self):
        return dict(name=self.name)


class Room:
    def __init__(self, id, owner_id):
        self.id: str = id
        self.owner_id: str = owner_id
        self.players: list[Client] = []

    def get_players_info(self, include_self=False):
        if include_self:
            return [p.get_info() for p in self.players]
        else:
            return [p.get_info() for p in self.players if p.id != request.sid]  # type: ignore

    def add_player(self, player):
        self.players.append(player)


class _ClientsManager:
    def __init__(self):
        self.__clients: dict[str, Client] = {}  # a dict of all connected clients

    def get_client(self) -> Client | None:
        return self.__clients.get(request.sid)  # type: ignore

    def new_client(self):
        id = request.sid  # type: ignore
        if id not in self.__clients:
            self.__clients[id] = Client(id)

    def remove_client(self):
        id = request.sid  # type: ignore
        if id in self.__clients:
            del self.__clients[id]


class _GameState:
    PLAYERS_LIMIT = 2

    def __init__(self, clients_manager: _ClientsManager):
        self.__rooms: dict[str, Room] = {}
        self.clients_manager = clients_manager

    def get_room(self, room_id) -> Room | None:
        return self.__rooms.get(room_id)

    def create_room(self, owner_id):
        room_id = shortuuid.uuid()

        # check for collisions
        if room_id in self.__rooms:
            room_id = shortuuid.uuid()

        self.__rooms[room_id] = Room(room_id, owner_id)

        client = ClientsManager.get_client()

        if not client:
            return

        client.owned_rooms.append(room_id)

        return room_id

    def remove_room(self, id) -> bool:
        try:
            del self.__rooms[id]
            return True
        except KeyError:
            return False

    def join_room(self, room_id: str, client: Client, role="player") -> Room | None:
        room = self.get_room(room_id)

        if room is None:
            return None

        if len(room.players) >= self.PLAYERS_LIMIT:
            return None

        room.add_player(client)

        return room

    def leave_room(self, room_id, client_id):
        room = self.get_room(room_id)
        client = self.clients_manager.get_client()

        if (room is None) or (client is None):
            return

        is_room_owner = room_id in client.owned_rooms

        if is_room_owner:
            self.remove_room(room_id)
            close_room(room_id)
        else:
            for i in range(len(room.players)):
                if room.players[i].id == client_id:
                    del room.players[i]
                    leave_room(room_id)
                    return

    def disconnect_player(self):
        for room in rooms():
            self.leave_room(room, request.sid)  # type: ignore


ClientsManager = _ClientsManager()
GameState = _GameState(ClientsManager)
