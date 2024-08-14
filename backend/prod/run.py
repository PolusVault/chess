# type: ignore
from src import create_app, socketio

app, _ = create_app()

if __name__ == "__main__":
    socketio.run(app)
