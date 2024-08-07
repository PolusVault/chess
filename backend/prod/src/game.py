# type: ignore

from flask import Blueprint, request
from flask_socketio import join_room
from . import socketio
from .core import GameState
from .utils import success, error

game = Blueprint("game", __name__)


@socketio.on("connect")
def connect():
    GameState.new_client(request.sid)


@socketio.on("disconnect")
def disconnect():
    GameState.disconnect_player(request.sid)


@socketio.on("create-game")
def create_room():
    room_id = GameState.create_room(request.sid)
    room = GameState.join_room(room_id, request.sid)

    if room is None:
        raise RuntimeError("unable to create room")

    return success(room_id)


@socketio.on("join-game")
def join_game(data):
    room_id = data["room_id"]
    room = GameState.join_room(room_id, request.sid)

    if room is None:
        raise RuntimeError("unable to join room")

    return success(room.get_players_info())


@socketio.on("leave-game")
def leave_game(data):
    room_id = data["room_id"]

    GameState.leave_room(room_id, request.sid)

    return success()


@socketio.on("make-move")
def make_move(data):
    room_id = data["room_id"]
    content = data["content"]

    socketio.emit("make-move", content, include_self=False, to=room_id)

    return success()


@socketio.on_error()  # Handles the default namespace
def error_handler(e):
    return error(str(e))
