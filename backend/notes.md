what is base64 encoding and why?
- a way of taking binary data and turning it into ASCII characters 
- it used in contexts where text are expected, not raw bytes


what is the size limit of a single websocket frame?


Websocket messages JSON formats:

codes:
0 = create a game,
1 = join a game,
2 = leave a game,
3 = spectate a game,
4 = chat message,

5 = make a chess move


Creating a game:
{
   type: int = 0,
}

Joining a game:
{
   type: int = 1,
   payload: string = "<game-code>"
}

Leaving a game:
{
   type: int = 2,
   payload: string = "<game-code>"
}

Making a chess move:
{
   type: int = 5,
   payload: string = "<move-notation>"
}






