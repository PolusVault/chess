import logging
import pytest
from .. import create_app, socketio

app, config = create_app(is_testing=True)

from ..game import IP_Limiter, RateLimiter, GameState


def test_create_game(caplog):
    caplog.set_level(logging.WARNING)

    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    assert client.is_connected() == True

    RateLimiter.disable()

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
    assert client.is_connected() == False

    RateLimiter.enable()
    GameState.reset()


def test_join_game(caplog):
    caplog.set_level(logging.WARNING)
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


def test_badinput():
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    ack = client.emit(
        "create-game", {"payload": {"color": "white", "name": "hieu"}}, callback=True
    )
    assert ack == []
    assert client.is_connected() == False
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    assert client.is_connected() == False  # banned, so wont connect again

    IP_Limiter.reset_test()

    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    ack = client.emit(
        "create-game",
        {
            "payload": {
                "color": "w",
                "name": "this is a very long long long long long name!!!!!!!",
            }
        },
        callback=True,
    )
    assert ack == []
    assert client.is_connected() == False
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    assert client.is_connected() == False  # banned, so wont connect again

    IP_Limiter.reset_test()

    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    ack = client.emit(
        "create-game", {"payload": {"color": "w", "name": "hieu"}}, callback=True
    )

    assert "success" in ack and "payload" in ack
    assert ack["success"] == True
    assert type(ack["payload"]) == str


def test_ratelimit():
    client = socketio.test_client(app, headers={"REMOTE_ADDR": "127.0.0.1"})
    client2 = socketio.test_client(
        app, headers={"REMOTE_ADDR": "some random ip address"}
    )

    for _ in range(100):
        if client.is_connected():
            client.emit(
                "make-move",
                {"payload": {"room_id": "id", "move": {"from": "a3", "to": "b1"}}},
            )
        else:
            break

    assert client.is_connected() == False
    with pytest.raises(RuntimeError):
        client.emit(
            "make-move",
            {"payload": {"room_id": "id", "move": {"from": "a3", "to": "b1"}}},
        )

    assert IP_Limiter.is_banned("127.0.0.1") == True
    assert client2.is_connected() == True

    client2.emit("create-game", {"payload": {"color": "b", "name": "hieu1"}})
    client2.emit(
        "make-move",
        {"payload": {"room_id": "id", "move": {"from": "a3", "to": "b1"}}},
    )
    client2.emit(
        "make-move",
        {"payload": {"room_id": "id", "move": {"from": "a3", "to": "b1"}}},
    )

    assert IP_Limiter.is_banned("some random ip address") == False
    assert client2.is_connected() == True
