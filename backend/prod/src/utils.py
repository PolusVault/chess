import os
from flask import request, current_app


def success(data=None):
    if data is not None:
        return {"success": True, "payload": data}
    else:
        return {"success": True}


def error(reason=None):
    return {"success": False, "reason": reason}


def is_prod_env():
    return os.environ.get("ENV") == "PROD"


# don't use this function outside of a request context
def get_ip():
    ip = None

    if current_app.config["TESTING"] == True:
        return request.headers["REMOTE_ADDR"]

    if request.environ.get("HTTP_X_FORWARDED_FOR") is None:
        ip = request.environ["REMOTE_ADDR"]
    else:
        ip = request.environ["HTTP_X_FORWARDED_FOR"]

    return ip
