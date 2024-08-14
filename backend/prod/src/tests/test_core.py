import pytest
from ..core import _GameState, Room, Client
from .. import create_app

_, config = create_app(is_testing=True)

GameState = _GameState(config)


def test_client():
    clientWithNoName = Client(0)
    clientWithName = Client(1, "Chess")

    assert clientWithNoName.id == 0 and clientWithNoName.name == "anonymous"
    assert clientWithName.id == 1 and clientWithName.name == "Chess"
    assert clientWithName.get_info() == {"name": "Chess"}


def test_room():
    player1 = Client(0)
    player2 = Client(1)

    room = Room(0, "foo")
    room.add_player(player1, "w")
    assert room.players["w"] == player1
    room.add_player(player2)
    assert room.players["b"] == player2
    assert room.is_full() == True

    room2 = Room(1, "bar")
    room2.add_player(player1)
    assert room2.players["w"] == player1
    room2.add_player(player2)
    assert room2.players["b"] == player2
    assert room2.is_empty() == False

    room3 = Room(2, "baz")
    assert room3.is_full() == False
    assert room3.is_empty() == True

    room = Room(3, "test")
    assert room.add_player(player1, "b") == True
    assert room.add_player(player2) == True

    assert room.player_ids == [0, 1]
    # same player can't join room again
    assert room.add_player(player1) == False

    assert room.get_opponent(0) == {"color": "w", "name": "anonymous"}
    assert room.get_opponent(1) == {"color": "b", "name": "anonymous"}
    assert room.get_player(1) == {"color": "w", "name": "anonymous"}
    assert room.get_player(0) == {"color": "b", "name": "anonymous"}

    room.remove_player(0)
    assert room.players["b"] == None
    assert room.is_full() == False
    room.remove_player(1)
    assert room.is_empty() == True
    assert room.player_ids == []


def test_GameState():
    assert GameState.get_client(0) == None

    client = GameState.new_client(0)
    client2 = GameState.new_client(1)
    assert GameState.get_client(0) == client
    assert GameState.get_client(1) == client2
    assert GameState.new_client(0) == None

    GameState.remove_client(0)
    assert GameState.get_client(0) == None
    assert GameState.get_client(1) == client2

    assert GameState.get_room(0) == None

    with pytest.raises(Exception):
        GameState.create_room(0, "foo")

    room_id = GameState.create_room(1, "bar")
    assert type(room_id) == str
    assert GameState.get_client(1).name == "bar"
    assert GameState.get_room(room_id).id == room_id

    assert GameState.remove_room("nonexistent id") == False
    assert GameState.remove_room(room_id) == True

    room_id = GameState.create_room(client2.id, "client name")
    with pytest.raises(Exception):
        GameState.join_room(1, "nonexistent client id")
    GameState.join_room(room_id, client2.id, "client2")

    client3 = GameState.new_client(3)
    room = GameState.join_room(room_id, client3.id, "baz")
    assert type(room) == Room and room.id == room_id
    assert GameState.get_client(3).name == "baz"
    assert GameState.get_room(room_id).is_full() == True
    assert room.player_ids == [1, 3]

    client4 = GameState.new_client(4)
    with pytest.raises(Exception):
        GameState.join_room(room_id, client4.id, "fuz")  # room is full
    with pytest.raises(Exception):
        GameState.join_room(
            "non existent room id", client4.id, "fuz"
        )  # room doesn't exist

    GameState.reset()
    client = GameState.new_client(0)
    client2 = GameState.new_client(1)
    room_id = GameState.create_room(client.id, "new client name")
    room = GameState.join_room(room_id, client.id, None, "b")
    room = GameState.join_room(room_id, client2.id, "new client2 name")

    with pytest.raises(Exception):
        GameState.leave_room("non existent room id", client.id)
    with pytest.raises(Exception):
        GameState.leave_room(room_id, "non existent client id")

    room, player = GameState.leave_room(room_id, client.id)
    assert room.id == room_id
    assert player["color"] == "b"
    assert player["name"] == "new client name"
    assert GameState.get_room(room_id).get_player(client.id) == None
    assert GameState.get_room(room_id).get_player(client2.id) == {
        "color": "w",
        "name": "new client2 name",
    }

    room, player = GameState.leave_room(room_id, client2.id)
    assert room.id == room_id
    assert player["color"] == "w"
    assert GameState.get_room(room.id) == None
