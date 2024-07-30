def success(data=None):
    return {"success": True, "data": data}


def error(reason=None):
    return {"status": False, "reason": reason}
