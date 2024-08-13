from .. import create_app, socketio
from ..core import GameState
import logging

app = create_app(is_testing=True)


def test_create_game(caplog):
    caplog.set_level(logging.DEBUG)
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    assert client.is_connected() == True

    ack = client.emit(
        "create-game", {"payload": {"color": "w", "name": "hieu"}}, callback=True
    )

    assert "success" in ack and "payload" in ack
    assert ack["success"] == True
    assert type(ack["payload"]) == str

    client.emit("create-game", {"payload": {"color": "b", "name": "hieu1"}})
    client.emit("create-game", {"payload": {"color": "w", "name": "hieu2"}})
    client.emit("create-game", {"payload": {"color": "b", "name": "hieu3"}})
    client.emit("create-game", {"payload": {"color": "w", "name": "hieu4"}})

    # create more than 5 rooms should error
    ack = client.emit(
        "create-game", {"payload": {"color": "w", "name": "hieu1"}}, callback=True
    )

    assert "success" not in ack and "payload" not in ack
    assert ack == []

    GameState.reset()


def test_join_game(caplog):
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    client2 = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})

    res = client.emit(
        "create-game", {"payload": {"color": "w", "name": "hieu"}}, callback=True
    )

    room_id = res["payload"]

    res = client2.emit(
        "join-game", {"payload": {"room_id": room_id, "name": "hieu2"}}, callback=True
    )

    assert "success" in res and "payload" in res
    assert res["success"] == True
    assert res["payload"] == {"name": "hieu", "color": "w"}

    received = client.get_received()
    assert len(received) == 1
    player = received[0]["args"][0]
    assert player == {"name": "hieu2", "color": "b"}

    GameState.reset()


def test_leave_game():
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    client2 = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})

    res = client.emit(
        "create-game", {"payload": {"color": "b", "name": "hieu"}}, callback=True
    )

    room_id = res["payload"]

    res = client2.emit(
        "join-game", {"payload": {"room_id": room_id, "name": "hieu2"}}, callback=True
    )

    res = client.emit("leave-game", {"payload": {"room_id": room_id}}, callback=True)
    assert res["success"] == True

    received = client2.get_received()
    assert len(received) == 1
    player = received[0]["args"][0]
    assert player == {"name": "hieu", "color": "b"}

    GameState.reset()


# def test_ratelimit(caplog):
#     caplog.set_level(logging.DEBUG)
#     client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
#     for i in range(10):
#         if client.is_connected():
#             client.emit(
#                 "make-move",
#                 {"payload": {"room_id": "id", "move": {"from": "a3", "to": "b1"}}},
#             )
# try:
# except:
#     if not client.is_connected():
#         print("NOT CONNECTED")
#         client.connect()
# client2 = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
# assert client2.is_connected() == False
