def success(data=None):
    return {"success": True, "data": data}


def error(reason=None):
    return {"success": False, "reason": reason}
