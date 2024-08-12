from flask import Blueprint, request
from flask_socketio import (
    emit,
    join_room,
    leave_room,
    close_room,
    disconnect,
    ConnectionRefusedError,
)
import logging
from . import socketio
from .core import GameState
from .utils import success, error, is_prod_env
from .limit import IP_Limiter

game = Blueprint("game", __name__)

logger = logging.getLogger("mysocketio")
handler = logging.StreamHandler()
formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
handler.setFormatter(formatter)
logger.addHandler(handler)

if is_prod_env():
    logger.setLevel(logging.WARNING)
else:
    logger.setLevel(logging.DEBUG)


def get_ip():
    ip = None

    if request.environ.get("HTTP_X_FORWARDED_FOR") is None:
        ip = request.environ["REMOTE_ADDR"]
    else:
        ip = request.environ["HTTP_X_FORWARDED_FOR"]

    return ip


@socketio.on("connect")
def connect():
    logger.info("new socket connection")

    ip = get_ip()

    if IP_Limiter.is_banned(ip):
        logger.info("banned: %s", ip)
        raise ConnectionRefusedError("banned")

    if IP_Limiter.handle_conn(ip) is False:
        logger.info("limit reached: %s", ip)
        raise ConnectionRefusedError("connection limit reached")

    try:
        GameState.new_client(request.sid)
    except Exception as e:
        logger.info("connection error: %s", str(e))
        raise ConnectionRefusedError("unable to connect, try again later")


@socketio.on("disconnect")
def disconnect_():
    logger.info("socket disconnect")
    GameState.disconnect_player(request.sid)

    ip = get_ip()
    IP_Limiter.handle_disconn(ip)

    logger.info("room count: %s", GameState.get_room_count())
    logger.info("clients count: %s", GameState.get_client_count())


@socketio.on("create-game")
def create_room(data):
    color = data["payload"]["color"]
    name = data["payload"]["name"]

    if color != "w" and color != "b" or len(color) >= 2:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid color")

    if len(name) >= 20:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid name")

    try:
        room_id = GameState.create_room(request.sid, name)
    except:
        disconnect()
        return

    room = GameState.join_room(room_id, request.sid, name, color)

    if room is None:
        logger.info("error creating room - name: %s , color: %s", name, color)
        raise Exception("unable to create room")

    join_room(room_id)

    logger.info("room count: %s", GameState.get_room_count())
    logger.info("clients count: %s", GameState.get_client_count())

    return success(room_id)


@socketio.on("join-game")
def join_game(data):
    room_id = data["payload"]["room_id"]
    name = data["payload"]["name"]

    if len(name) >= 20:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid color")

    if len(room_id) >= 10:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid id")

    room = GameState.join_room(room_id, request.sid, name)

    if room is None:
        logger.info("error joining room room_id: %s , name: %s", room_id, name)
        raise Exception("unable to join room")

    join_room(room_id)

    # let the other player know we've joined
    emit(
        "opponent-connected",
        room.get_player(request.sid),
        to=room.id,
        include_self=False,
    )

    logger.info("room count: %s", GameState.get_room_count())
    logger.info("clients count: %s", GameState.get_client_count())

    return success(room.get_opponent(request.sid))


@socketio.on("leave-game")
def leave_game(data):
    room_id = data["payload"]["room_id"]

    if len(room_id) >= 10:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid id")

    room = GameState.leave_room(room_id, request.sid)

    leave_room(room.id)

    if room.is_empty():
        close_room(room.id)
    else:
        emit(
            "opponent-disconnected",
            room.get_player(request.sid),
            to=room.id,
            include_self=False,
        )

    logger.info("room count: %s", GameState.get_room_count())
    logger.info("clients count: %s", GameState.get_client_count())

    return success()


@socketio.on("make-move")
def make_move(data):
    room_id = data["payload"]["room_id"]
    move = data["payload"]["move"]

    if len(room_id) >= 10:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("invalid id")

    try:
        f = move["from"]
        to = move["to"]
        promo = ""
        if "promotion_piece" in move:
            promo = move["promotion_piece"]

        if len(f) >= 3 or len(to) >= 3 or len(promo) >= 3:
            raise Exception("move")
    except:
        IP_Limiter.ban(get_ip())
        disconnect()
        raise Exception("move")

    emit("make-move", move, include_self=False, to=room_id)

    logger.info("move: %s", move)

    return success()


@socketio.on_error()  # Handles the default namespace
def error_handler(e):
    # disconnect()
    logger.debug(e)
    return error(str(e))
