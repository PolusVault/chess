what is base64 encoding and why?
- a way of taking binary data and turning it into ASCII characters 
- it used in contexts where text are expected, not raw bytes


what is the size limit of a single websocket frame?


Websocket messages JSON formats:

Creating a game:
{
   type: string = "CREATE",
}

Joining a game:
{
   type: string = "JOIN",
   payload: string = "<game-code>"
}

Leaving a game:
{
   type: string = "LEAVE",
   payload: string = "<game-code>"
}

Making a chess move:
{
   type: string = "MOVE",
   payload: string = "<move-notation>"
}






