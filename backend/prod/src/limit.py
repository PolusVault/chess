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
        if len(self.banned) <= 10000:
            self.banned.append(ip)
        else:
            pass

    def is_banned(self, ip):
        return ip in self.banned


IP_Limiter = _IP_Limiter()


def rate_limiter():
    pass
