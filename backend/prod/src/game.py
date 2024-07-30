# type: ignore

from flask import Blueprint, request
from flask_socketio import leave_room, join_room, close_room
from . import socketio
from .core import GameState, Client, ClientsManager
from .utils import success, error

game = Blueprint("game", __name__)


@socketio.on("connect")
def connect():
    ClientsManager.new_client()


@socketio.on("disconnect")
def disconnect():
    GameState.disconnect_player()
    ClientsManager.remove_client()


@socketio.on("create-game")
def create_room():
    room_id = GameState.create_room(request.sid)
    client = ClientsManager.get_client()

    room = GameState.join_room(room_id, client)

    if room and room_id:
        join_room(room_id)
        return success(room_id)
    else:
        return error("unable to create room")


@socketio.on("join-game")
def join_game(data):
    room_id = data["room_id"]
    client = ClientsManager.get_client()

    room = GameState.join_room(room_id, client)

    if room:
        join_room(room_id)
        return success(room.get_players_info())
    else:
        return error("unable to join room")


@socketio.on("leave-game")
def leave_game(data):
    room_id = data["room_id"]
    room = GameState.get_room(room_id)

    if room is None:
        return error("unable to leave room")

    GameState.leave_room(room_id, request.sid)

    return success()


@socketio.on("make-move")
def make_move(data):
    room_id = data["room_id"]
    content = data["content"]

    socketio.emit("make-move", content, include_self=False, to=room_id)

    return success()
