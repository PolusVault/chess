from flask import Blueprint

http = Blueprint(
    "http", __name__, static_folder="../dist/assets", static_url_path="/assets"
)


@http.route("/heartbeat")
def heartbeat():
    return {"status": "healthy"}


@http.route("/", defaults={"path": ""})
@http.route("/<path:path>")
def catch_all(path):
    return http.send_static_file("index.html")
