# MaCh Protocol

This document is a description of the protocol used by clients to communicate with servers over a connection.

The protocol is currently in an alpha state and is subject to dramatic changes.

## Format

The MaCh protocol is message oriented. The most basic unit of communication is a Message, a complete piece of either text or binary data, which must be specified in the header of the Message in a manner unspecified here.

The canonical way to implement the MaCh protocol is over websockets, which provide a ready implementation of the Message and the header specifying whether data is binary or text as described above.

Messages of the text format must be valid JSON.

Binary messages may vary in encoding depending on previous messages.

## Protocol

### Handshake

After a connection is formed, before any further action takes place, a handshake must be performed between the client and the server. The handshake begins with the server, who will send a text message with the contents:

```
{
	"msg": "Handshake",
	"versions": <versions>,
}
```

where `<versions>` is an array of integers representing each version of the MaCh protocol supported by the server.

The client must then respond with either

```
{
	"msg": "HandshakeOk",
	"version": <version>
}
```

where `<version>` is a version integer that was advertised by the server in the initial handshake message, or

```
{
	"msg": "HandshakeFailure",
	"reason": <reason>
}
```

to indicate some sort of failure or rejection of connection. `<reason>` must be string. The value of `<reason>` may be anything, but certain key values may be recognized specially by the server. A value of `"unsupported"` means that the client was not compatible with any of the protocl versions the server advertised.

If the failure response is given, then the connection will be closed and no further messages will be sent or received.

### IDs

In order to allow clients and servers to specify which messages they are responding to, some messages will include IDs. An ID is a signed 32 bit integer that is unique and meaningful only to the current connection. The client and server must each implement a system that allows them to create IDs unique to a connection at will for use in requests. To prevent conflicts caused the client and the server creating equal IDs simultaneously, IDs created by the server must always be negative, and IDs created by the client must always be positive. The ID `0` is reserved and must not be used by the client or server as a regular ID.

### Game Entry

To join a game that already exists, the client needs an invite token from an external source. The invite token is an eight character string containing only characters from [A-Za-z0-9]. Once the client that wishes to join a game has an invite token, it can send the following message to request to join the game:

```
{
	"msg": "JoinGameRequest",
	"id": <new_id>,
	"invite_token": <invite_token>,
	"color": <color>
}
```

### Game Creation

To create a game that does not exist yet, the following message should be sent by a client to the server:

```
{
	"msg": "CreateGameRequest",
	"id": <new_id>,
	"color": <color>
}
```

`<color>` indicates the preferred color for the player creating the game and must be either `"black"`, `"white"`, or `"random"` to indicate that no the player should be assigned a color randomly by the server.

In order for other players to join the newly created game, an invite token must be created for the game, with the following request:

```
{
	"msg": "GetInviteTokenRequest",
	"id": <new_id>,
	"game_id": <game_id>
}
```

In this request, `<new_id>` is a newly created client id that will be used by the server to indicate which request is being replied to. `<game_id>` is the id of the 