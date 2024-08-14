import time
from ..limit import _IP_Limiter, _RateLimiter
from .. import create_app

_, config = create_app(is_testing=True)  # to load the app config

IP_Limiter = _IP_Limiter(config, 2)
RateLimiter = _RateLimiter(IP_Limiter, config)


def test_ip_limiter():
    assert IP_Limiter.handle_conn("test ip") == True
    assert IP_Limiter.handle_conn("test ip") == True
    IP_Limiter.handle_disconn("test ip")
    assert IP_Limiter.connections["test ip"] == 1
    IP_Limiter.handle_disconn("test ip")
    assert "test ip" not in IP_Limiter.connections

    assert IP_Limiter.handle_conn("test ip") == True
    assert IP_Limiter.handle_conn("test ip") == True
    assert IP_Limiter.handle_conn("test ip") == False
    assert IP_Limiter.is_banned("test ip") == True

    IP_Limiter.ban("foo")
    assert IP_Limiter.is_banned("foo") == True


def test_ratelimiter():
    RateLimiter.set("test ip", 2)
    assert "count" in RateLimiter.get("test ip")

    time.sleep(2)

    assert RateLimiter.get("test ip") == None

    RateLimiter.set("test ip", 2)
    for _ in range(100):
        RateLimiter.incr("test ip")

    assert RateLimiter.get("test ip")["count"] > RateLimiter.max_req_count

    time.sleep(2)

    RateLimiter.set("test ip", 2)
    for _ in range(5):
        time.sleep(2)

        req = RateLimiter.get("test ip")

        if req is None:
            RateLimiter.set("test ip", 2)

        RateLimiter.incr("test ip")

    assert RateLimiter.get("test ip")["count"] <= RateLimiter.max_req_count


def test_ratelimiter_backgroundtask():
    RateLimiter.set("test ip 1", 2)
    RateLimiter.set("test ip 2", 3)
    RateLimiter.set("test ip 3", 1)
    RateLimiter.set("test ip 4", 4)

    time.sleep(8)

    assert RateLimiter.get_without_cleanup("test ip 1") == None
    assert RateLimiter.get_without_cleanup("test ip 2") == None
    assert RateLimiter.get_without_cleanup("test ip 3") == None
    assert RateLimiter.get_without_cleanup("test ip 4") == None
