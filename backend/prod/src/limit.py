import time
from functools import wraps
from flask import request
from flask_socketio import disconnect
import logging
from .utils import get_ip


class _IP_Limiter:
    def __init__(self, limit=10):
        self.limit = limit
        self.connections = {}
        self.banned = []

    def handle_conn(self, ip):
        """
        this method returns False if a connection exceed the limit
        """
        if len(self.connections) >= 500:
            return False

        if ip in self.connections:
            if self.connections[ip] >= self.limit:
                if len(self.banned) <= 10000:
                    self.banned.append(ip)
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
            pass

    def ban(self, ip):
        self.banned.append(ip)

    def is_banned(self, ip):
        return ip in self.banned


IP_Limiter = _IP_Limiter()


logger = logging.getLogger("limiter")
handler = logging.StreamHandler()
formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
handler.setFormatter(formatter)
logger.addHandler(handler)


class _RateLimiter:
    def __init__(self):
        # 3 req / s
        self.max_req_count = 10
        self.__requests = {}

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
        for key, val in self.__requests.items():
            if val["creation_time"] - curr_time >= val["time_window"]:
                logger.debug("bye")
                del self.__requests[key]

    def limit(self, handler):
        @wraps(handler)
        def with_ratelimit(*args, **kwargs):
            ip = get_ip()

            req = self.get(ip)

            if req is not None and req["count"] > self.max_req_count:
                logger.warning("rate limit exceeded")
                disconnect()
                IP_Limiter.ban(ip)
                return

            if req is None:
                self.set(ip)
            else:
                self.incr(ip)

            return handler(*args, **kwargs)

        return with_ratelimit


RateLimiter = _RateLimiter()
