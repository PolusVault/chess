import os
import tomllib
from flask import Flask
from flask_socketio import SocketIO
import logging


# TODO: change cors before deploying!!!!
socketio = SocketIO(path="/chess/socket", cors_allowed_origins="*")
logger = logging.getLogger("werkzeug")
logger.setLevel(logging.WARNING)


def create_app(is_testing=False):
    app = Flask(__name__, instance_relative_config=True)

    app.config.from_file("config.dev.toml", load=tomllib.load, text=False)
    app.config.update(TESTING=is_testing)
    app.config.from_file("config.prod.toml", load=tomllib.load, text=False, silent=True)

    try:
        os.makedirs(app.instance_path)
    except OSError:
        pass

    from .http import http
    from .game import game

    app.register_blueprint(http)
    app.register_blueprint(game)

    socketio.init_app(app)

    return app
