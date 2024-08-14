import logging
from .utils import is_prod_env

formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
handler = logging.StreamHandler()
handler.setFormatter(formatter)

limit_logger = logging.getLogger("limiter")
socket_logger = logging.getLogger("mysocketio")

loggers = [limit_logger, socket_logger]

for logger in loggers:
    logger.addHandler(handler)

    if is_prod_env():
        logger.setLevel(logging.WARNING)
    else:
        logger.setLevel(logging.DEBUG)
