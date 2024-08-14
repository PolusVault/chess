import os
import tomllib
from flask import Flask
from flask_socketio import SocketIO
import logging
from .utils import is_prod_env

cors = []
if is_prod_env():
    cors = []
else:
    cors = "*"

socketio = SocketIO(path="/chess/socket", cors_allowed_origins=cors)
logger = logging.getLogger("werkzeug")
logger.setLevel(logging.WARNING)

config = None


def create_app(is_testing=False):

    app = Flask(__name__, instance_relative_config=True)

    app.config.update(TESTING=is_testing)
    app.config.from_file("config.dev.toml", load=tomllib.load, text=False)
    app.config.from_file("config.test.toml", load=tomllib.load, text=False, silent=True)
    app.config.from_file("config.prod.toml", load=tomllib.load, text=False, silent=True)
    global config
    config = app.config

    try:
        os.makedirs(app.instance_path)
    except OSError:
        # dir already exists
        pass

    # from .core import GameState
    # from .limit import _IP_Limiter, _RateLimiter

    # # TODO: figure out a better way to handle configs
    # GameState.CONN_LIMIT = int(app.config["CONN_LIMIT"])
    # _IP_Limiter.CONN_LIMIT = GameState.CONN_LIMIT
    # _IP_Limiter.BAN_LIMIT = int(app.config["BAN_LIMIT"])
    # _RateLimiter.RATE_LIMIT_BACKGROUND_INTERVAL = int(
    #     app.config["RATE_LIMIT_BACKGROUND_INTERVAL"]
    # )

    from .http import http
    from .game import game

    app.register_blueprint(game)

    # nginx will handle static contents in production
    if not is_prod_env():
        app.register_blueprint(http)

    socketio.init_app(app)

    return app, config
