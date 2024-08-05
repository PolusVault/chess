import os
import tomllib
from flask import Flask
from flask_socketio import SocketIO
import logging

socketio = SocketIO(path="/chess/socket")
logger = logging.getLogger("werkzeug")
logger.setLevel(logging.ERROR)

def create_app(test_config=None):
    app = Flask(__name__, instance_relative_config=True)
    # app.config.from_prefixed_env()
    #
    # if app.config["ENV"] == "dev":
    #     if test_config is None:
    #        app.config.from_file("config.dev.toml", load=tomllib.load, text=False)
    #     else:
    #         app.config.from_mapping(test_config)
    # else:
    # app.config.from_file("config.prod.toml", load=tomllib.load, text=False)

    app.config.from_file("config.dev.toml", load=tomllib.load, text=False)

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
