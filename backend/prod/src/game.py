from flask import Blueprint, request
from flask_socketio import (
    emit,
    join_room,
    leave_room,
    close_room,
    disconnect,
    ConnectionRefusedError,
)
from . import socketio, config
from .core import _GameState
from .utils import success, get_ip
from .limit import _IP_Limiter, _RateLimiter
from .loggers import socket_logger


game = Blueprint("game", __name__)

IP_Limiter = _IP_Limiter(config)
RateLimiter = _RateLimiter(IP_Limiter, config)
GameState = _GameState(config)


def ban():
    IP_Limiter.ban(get_ip())
    disconnect()
    # raise Exception(e)


@socketio.on("connect")
def connect():
    socket_logger.debug("new socket connection")

    ip = get_ip()

    if IP_Limiter.is_banned(ip) or (IP_Limiter.handle_conn(ip) is False):
        # logger.debug("banned: %s", ip)
        raise ConnectionRefusedError()

    try:
        GameState.new_client(request.sid)
    except Exception as e:
        socket_logger.debug("connection error: %s", str(e))
        raise ConnectionRefusedError()


@socketio.on("disconnect")
def disconnect_():
    socket_logger.debug("socket disconnect")

    GameState.disconnect_player(request.sid)
    IP_Limiter.handle_disconn(get_ip())

    socket_logger.debug("room count: %s", GameState.get_room_count())
    socket_logger.debug("clients count: %s", GameState.get_client_count())


@socketio.on("create-game")
@RateLimiter.limit
def create_game(data):
    color = data["payload"]["color"]
    name = data["payload"]["name"]

    if color != "w" and color != "b" or len(color) >= 2 or len(name) >= 20:
        ban()
        return

    try:
        room_id = GameState.create_room(request.sid, name)
        GameState.join_room(room_id, request.sid, name, color)

        # if room is None:
        #     logger.debug("error creating room - name: %s , color: %s", name, color)
        #     # raise Exception("unable to create room")
        #     disconnect()
        #     return False

        join_room(room_id)

        socket_logger.debug("room count: %s", GameState.get_room_count())
        socket_logger.debug("clients count: %s", GameState.get_client_count())

        return success(room_id)
    except Exception as e:
        socket_logger.debug(e)
        disconnect()
        return


@socketio.on("join-game")
@RateLimiter.limit
def join_game(data):
    room_id = data["payload"]["room_id"]
    name = data["payload"]["name"]

    if len(name) >= 20 or len(room_id) >= 10:
        ban()
        return

    try:
        room = GameState.join_room(room_id, request.sid, name)

        # if room is None:
        #     logger.debug("error joining room room_id: %s , name: %s", room_id, name)
        #     raise Exception("unable to join room")

        join_room(room_id)

        # let the other player know we've joined
        emit(
            "opponent-connected",
            room.get_player(request.sid),
            to=room.id,
            include_self=False,
        )

        socket_logger.debug("room count: %s", GameState.get_room_count())
        socket_logger.debug("clients count: %s", GameState.get_client_count())

        return success(room.get_opponent(request.sid))
    except:
        disconnect()
        return


@socketio.on("leave-game")
@RateLimiter.limit
def leave_game(data):
    room_id = data["payload"]["room_id"]

    if len(room_id) >= 10:
        ban()

    try:
        room, player = GameState.leave_room(room_id, request.sid)

        leave_room(room.id)

        if room.is_empty():
            close_room(room.id)
        else:
            emit(
                "opponent-disconnected",
                player,
                to=room.id,
                include_self=False,
            )

        socket_logger.debug("room count: %s", GameState.get_room_count())
        socket_logger.debug("clients count: %s", GameState.get_client_count())

        return success()
    except:
        # disconnect()
        # this operation should be error free, if it isn't, ban
        ban()
        return


@socketio.on("make-move")
@RateLimiter.limit
def make_move(data):
    room_id = data["payload"]["room_id"]
    move = data["payload"]["move"]

    if len(room_id) >= 10:
        ban()

    f = move["from"]
    to = move["to"]
    promo = ""
    if "promotion_piece" in move:
        promo = move["promotion_piece"]

    if len(f) >= 3 or len(to) >= 3 or len(promo) >= 3:
        ban()
        return

    emit("make-move", move, include_self=False, to=room_id)

    socket_logger.debug("move: %s", move)
    # return success()


# @socketio.on_error()  # Handles the default namespace
# def error_handler(e):
#     # disconnect()
#     logger.debug(e)
#     return error(str(e))
