import time
from functools import wraps
from flask_socketio import disconnect
from apscheduler.schedulers.background import BackgroundScheduler
from .utils import get_ip
from .loggers import limit_logger


class _IP_Limiter:
    CONN_LIMIT = None
    BAN_LIMIT = None

    def __init__(self, config, limit=10):
        self.limit = limit
        self.connections = {}
        self.banned = []

        _IP_Limiter.CONN_LIMIT = int(config["CONN_LIMIT"])
        _IP_Limiter.BAN_LIMIT = int(config["BAN_LIMIT"])

    def handle_conn(self, ip):
        """this method returns False if a connection exceed the limit"""
        if len(self.connections) >= _IP_Limiter.CONN_LIMIT:
            return False

        if len(self.banned) >= _IP_Limiter.BAN_LIMIT:
            return False

        if ip in self.connections:
            if self.connections[ip] >= self.limit:
                self.ban(ip)
                return False
            self.connections[ip] += 1
        else:
            self.connections[ip] = 1

        return True

    def handle_disconn(self, ip):
        if ip in self.connections:
            self.connections[ip] = max(0, self.connections[ip] - 1)
            if self.connections[ip] == 0:
                del self.connections[ip]
        else:
            # this should never happen
            raise RuntimeError("disconnecting nonexistent IP address")

    def ban(self, ip):
        self.banned.append(ip)

    def is_banned(self, ip):
        return ip in self.banned

    # for testing
    def reset_test(self):
        self.connections = {}
        self.banned = []


class _RateLimiter:
    RATE_LIMIT_BACKGROUND_INTERVAL = None
    MAX_REQ_COUNT = None

    def __init__(self, ip_limiter, config):
        self.ip_limiter = ip_limiter
        # self.max_req_count = max_req_count  # req / s
        self.__requests = {}
        self.disabled = False

        _RateLimiter.RATE_LIMIT_BACKGROUND_INTERVAL = int(
            config["RATE_LIMIT_BACKGROUND_INTERVAL"]
        )
        _RateLimiter.MAX_REQ_COUNT = int(config["MAX_REQ_COUNT"])

        scheduler = BackgroundScheduler()
        scheduler.add_job(
            self.cleanup,
            "interval",
            seconds=_RateLimiter.RATE_LIMIT_BACKGROUND_INTERVAL,
        )
        scheduler.start()

    def get(self, key):
        self.cleanup()
        return self.__requests.get(key)

    def set(self, key, time_in_sec=1):
        self.__requests[key] = {
            "time_window": time_in_sec,
            "count": 0,
            "creation_time": time.time(),
        }

    def incr(self, key):
        if key in self.__requests:
            self.__requests[key]["count"] = self.__requests[key]["count"] + 1

    # this might be slow
    # delete entries that has lived past their specified time
    def cleanup(self):
        curr_time = time.time()
        dirty = []
        for key, val in self.__requests.items():
            if curr_time - val["creation_time"] >= val["time_window"]:
                limit_logger.debug("bye")
                # del self.__requests[key]
                dirty.append(key)

        for k in dirty:
            del self.__requests[k]

    # --- TESTING METHODS, DO NOT USE ---
    def disable(self):
        self.disabled = True

    def enable(self):
        self.disabled = False

    def get_without_cleanup(self, key):
        return self.__requests.get(key)

    def limit(self, handler):
        @wraps(handler)
        def with_ratelimit(*args, **kwargs):
            if self.disabled:
                limit_logger.warning("disableddddddd")
                return handler(*args, **kwargs)

            ip = get_ip()

            req = self.get(ip)

            if req is not None and req["count"] > _RateLimiter.MAX_REQ_COUNT:
                limit_logger.warning("rate limit exceeded")
                limit_logger.warning(req)
                disconnect()
                self.ip_limiter.ban(ip)
                return

            if req is None:
                self.set(ip)
            else:
                self.incr(ip)

            return handler(*args, **kwargs)

        return with_ratelimit
