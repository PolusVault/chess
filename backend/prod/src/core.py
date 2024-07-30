import shortuuid
from flask import request
from flask_socketio import rooms, leave_room, close_room, join_room


class Client:
    def __init__(self, id, name="Anonymous"):
        self.id = id
        self.name = name
        # self.owned_rooms = []

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


class _GameState:
    PLAYERS_LIMIT = 2

    def __init__(self):
        self.__rooms: dict[str, Room] = {}
        self.__clients: dict[str, Client] = {}

    def get_client(self, id) -> Client | None:
        return self.__clients.get(id)

    def new_client(self, id):
        if id not in self.__clients:
            self.__clients[id] = Client(id)

    def remove_client(self, id):
        if id in self.__clients:
            del self.__clients[id]

    def get_room(self, room_id) -> Room | None:
        return self.__rooms.get(room_id)

    def create_room(self, client_id):
        """
        create a new room
        @param client_id: str - the id of the requesting client
        @return room_id: str - the id of the room created
        """
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise RuntimeError("create room: client doesn't exist")

        room_id = shortuuid.uuid()

        # check for collisions
        if room_id in self.__rooms:
            room_id = shortuuid.uuid()

        self.__rooms[room_id] = Room(room_id, client_id)

        return room_id

    def remove_room(self, id) -> bool:
        """
        attemp to delete a room, it provides no authorization so do not use it outside of this class
        @param id: str - the room id
        @return True if successful, False otherwise
        """
        try:
            del self.__rooms[id]
            return True
        except KeyError:
            return False

    def join_room(self, room_id, client_id, role="player") -> Room | None:
        """
        join an existing room
        @param room_id: str - the room id
        @param client_id: str - the client id
        @return Room (the room information) if the room exist, None if the room doesn't exist or is full
        """
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise RuntimeError("join room: client doesn't exist")

        room = self.get_room(room_id)

        if room is None:
            return None

        if len(room.players) >= self.PLAYERS_LIMIT:
            return None

        room.add_player(client)

        join_room(room_id)

        return room

    def leave_room(self, room_id, client_id):
        """
        leave an existing room
        @param room_id: str - the room id
        @param client_id: str - the client id
        @return None
        """
        room = self.get_room(room_id)
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise RuntimeError("leave room: client doesn't exist")
        elif room is None:
            raise RuntimeError("leave room: room doesn't exist")

        for i in range(len(room.players)):
            if room.players[i].id == client_id:
                del room.players[i]
                leave_room(room_id)
                break

        if len(room.players) == 0:
            self.remove_room(room_id)
            close_room(room_id)

    def disconnect_player(self, client_id):
        for room_id in rooms():
            if room_id == client_id:  # skip the default room
                continue
            self.leave_room(room_id, client_id)

        self.remove_client(client_id)


GameState = _GameState()
