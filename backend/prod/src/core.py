import shortuuid

from flask_socketio import rooms, leave_room, emit


class Client:
    def __init__(self, id, name="anonymous"):
        self.id = id
        self.name = name
        self.rooms_count = 0
        # self.owned_rooms = []

    def get_info(self):
        return dict(name=self.name)


class Room:
    def __init__(self, id, owner_id):
        self.id: str = id
        self.owner_id: str = owner_id
        self.player_ids = []
        self.players = {"w": None, "b": None}

    def get_player(self, id):
        for c, p in self.players.items():
            if p is not None and p.id == id:
                info = p.get_info()
                info["color"] = c
                return info
        return None

    def get_opponent(self, client_id):
        for c, p in self.players.items():
            if p is not None and p.id != client_id:
                info = p.get_info()
                info["color"] = c
                return info
        return None

    def is_full(self):
        for p in self.players.values():
            if p is None:
                return False

        return True

    def is_empty(self):
        for p in self.players.values():
            if p is not None:
                return False

        return True

    def add_player(self, player, color=None):
        if player.id in self.player_ids:
            return False

        if color is None:
            # assign the player to the first color available
            for c, p in self.players.items():
                if p is None:
                    self.players[c] = player
                    self.player_ids.append(player.id)
                    return True
        else:
            if self.players[color] is None:
                self.player_ids.append(player.id)
                self.players[color] = player
                return True

        return False

    def remove_player(self, id):
        for c, p in self.players.items():
            if p is not None and p.id == id:
                p.rooms_count = max(0, p.rooms_count - 1)
                self.players[c] = None
                self.player_ids.remove(id)


class _GameState:
    # LIMIT = int(config["CONN_LIMIT"])
    CONN_LIMIT = None  # this will be set properly once the app is loaded
    ROOMS_LIMIT = None

    def __init__(self, config):
        self.__rooms: dict[str, Room] = {}
        self.__clients: dict[str, Client] = {}

        _GameState.CONN_LIMIT = int(config["CONN_LIMIT"])
        _GameState.ROOMS_LIMIT = int(config["ROOMS_LIMIT"])

    def get_client(self, id) -> Client | None:
        return self.__clients.get(id)

    def new_client(self, id) -> Client | None:
        if len(self.__clients) >= _GameState.CONN_LIMIT:
            raise Exception("player limit exceeded")

        if id not in self.__clients:
            client = Client(id)
            self.__clients[id] = client
            return client

    def remove_client(self, id):
        if id in self.__clients:
            del self.__clients[id]

    def get_room(self, room_id) -> Room | None:
        return self.__rooms.get(room_id)

    def create_room(self, client_id, name=None) -> str:
        """create a new room

        :param client_id: str - the id of the requesting client
        :param name: str - the name of the client, if any
        :return room_id: str - the id of the room created
        """
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise Exception("create room: client doesn't exist")

        if client.rooms_count >= _GameState.ROOMS_LIMIT:
            raise Exception("rooms limit exceeded")

        if name is not None:
            client.name = name

        room_id = shortuuid.uuid()[:6]

        # check for collisions
        if room_id in self.__rooms:
            room_id = shortuuid.uuid()[:6]

        self.__rooms[room_id] = Room(room_id, client_id)
        client.rooms_count += 1

        return room_id

    def remove_room(self, id) -> bool:
        """attemp to delete a room, it provides no authorization so do not use it outside of this class

        :param id: str - the room id
        :return bool: True if successful, False otherwise
        """
        try:
            del self.__rooms[id]
            return True
        except KeyError:
            return False

    def join_room(
        self, room_id, client_id, name=None, color=None, role="player"
    ) -> Room:
        """join an existing room

        :param room_id: str - the room id
        :param client_id: str - the client id
        :return room: the room information
        """
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise Exception("join room: client doesn't exist")

        if name is not None:
            client.name = name

        room = self.get_room(room_id)

        if room is None or room.is_full():
            raise Exception("unable to join room")
            # return None

        # this condition should never happens
        if room.add_player(client, color) is False:
            raise Exception("unable to join room")

        return room

    def leave_room(self, room_id, client_id):
        """leave an existing room

        :param room_id: str - the room id
        :param client_id: str - the client id
        :return tuple: room info, player info
        """
        room = self.get_room(room_id)
        client = self.get_client(client_id)

        if client is None:  # the client MUST exist at this point
            raise Exception("leave room: client doesn't exist")
        elif room is None:
            raise Exception("leave room: room doesn't exist")

        # get player info before we remove them from the room
        player = room.get_player(client_id)
        if player is None:
            raise Exception("why are you none?")

        room.remove_player(client_id)

        if room.is_empty():
            self.remove_room(room_id)

        return (room, player)

    # don't use this method in tests
    def disconnect_player(self, client_id):
        for room_id in rooms():
            if room_id == client_id:  # skip the default room
                continue
            room, player = self.leave_room(room_id, client_id)
            emit(
                "opponent-disconnected",
                player,
                to=room.id,
                include_self=False,
            )
            leave_room(room_id)

        self.remove_client(client_id)

    def reset(self):
        self.__rooms = {}
        self.__clients = {}

    def get_room_count(self):
        return len(self.__rooms)

    def get_client_count(self):
        return len(self.__clients)


# GameState = _GameState()
