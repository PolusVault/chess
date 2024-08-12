import os


def success(data=None):
    return {"success": True, "payload": data}


def error(reason=None):
    return {"success": False, "reason": reason}


def is_prod_env():
    return os.environ.get("ENV") == "PROD"
