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


# @http.get("/")
# def chess():
#     return send_from_directory("../dist", "index.html")


# @http.get("/assets/<path:filename>")
# def assets(filename):
#     return send_from_directory("../dist/assets", filename)


# @http.post("/create")
# def create_game():
#     room_id = GameState.create_room()
#     return success(room_id)


# @http.get("/join")
# def join_game():
#     room_id = request.args.get("room_id")

#     if room_id is not None:
#         print("join room " + room_id)

#     return success(room_id)
