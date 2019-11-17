# Ability
## Architecture
   ![architecture](https://www.processon.com/view/link/5dc79055e4b0bd68d80ed96a)

## The Protocal
   1. Connection
      [sequence diagram](https://www.websequencediagrams.com/?lz=dGl0bGUgQ2xpZW50IENvbm5lY3QgV3NTZXJ2ZXIKCgATBi0tPgALCDogUmVxdWVzdCB3ZWJzb2NrZXQgdXJsCgArCC0tPkFwcAAkCGF1dGggYnkgaGVhZGVycwoAEgkARQ1hcHByb3ZlIG9yIGFib3J0AEEMAIEUBjogdXBncmFkAB8FcmVqZWN0Cg&s=default)

   2. Command
      1. Subscribe
         [sequence diagram](https://www.websequencediagrams.com/?lz=dGl0bGUgQ2xpZW50IFN1YnNjcmliZSBDaGFubmVsCgoAFAYtLT5Xc1NlcnZlcjogY29tbWFuZD1zACYILGlkZW50aWZpZXI9dGhlLQAFCgoALggtLT5BcHAAOghzZW5kIAA2CQpub3RlIHJpZ2h0IG9mIAAdC2NyZWF0ZSBjAIEHBiBvYmplY3QKAEAJAIEJDXJlcHkgd2l0aCB0cmFuc21pc3Npb25zAHYMAIFgBgAmBWwAJgYAHRA&s=default)
      2. Unsubscribe
      3. Message
         [sequence diagram](https://www.websequencediagrams.com/?lz=dGl0bGUgQ2xpZW50IENhbGwgQXBwU2VydmVyIENoYW5uZWwncyBNZXRob2QgdG8gU3RhcnQgU3RyZWFtCgoAMgYtLT5XcwAuBjogY29tbWFuZD1tZXNzYWdlLGlkZW50aWZpZXI9dGhlLWMATgYsIGFjdGlvbj1mb2xsb3cKADgILS0-AHcJOiBzZW5kIHRoZSAASAVhZ2UKbm90ZSByaWdodCBvZgCBIQoKAIE1BXRoZSAAWQcncyAAVAYgbQCBNgUsCkluAEUFAAgOIHdlIGNhbiBjYWxsIGBzAIFRBV9mcm9tYCB0byBzAIFnBQAQBgplbmQgbm90ZQoAghcJAIFqDXJlcHkgd2l0aCB0cmFuc21pc3Npb25zIGFuZAA7B3MAgS4PAIIpCnNhdmUAYAcAcgV1YnNjcmlidG9yIHJlbGF0aW9uc2hpcACCGQwAgysGAHIFbAByBgBqDwoK&s=default)
   3. Send message to channel
      [sequence diagram](https://www.websequencediagrams.com/?lz=dGl0bGUgQ2xpZW50IGNhbGwgQXBwU2VydmVyIENoYW5uZWwncyBtZXRob2QgdG8gc3RhcnQgc3RyZWFtCgoAMgYtLT5XcwAuBjogY29tbWFuZD1tZXNzYWdlLGlkZW50aWZpZXI9dGhlLWMATgYsIGFjdGlvbj1mb2xsb3cKADgILS0-AHcJOiBzZW5kIHRoZSAASAVhZ2UKbm90ZSByaWdodCBvZgCBIQoKQ2FsbAAlBQBZBydzIABUBgCBNAcsCkluAEUFAAgOIHdlIGNhbgCBcwZgAIFQBl9mcm9tYACBWxFlbmQgbm90ZQoAghcJAIFqDXJlcHkgd2l0aCB0cmFuc21pc3Npb25zIGFuZACCJwdzAIEuDwCCKQpzYXZlAIJMBwCCXgV1YnNjcmlidG9yIHJlbGF0aW9uc2hpcACCGQwAgysGAHIFbAByBgBqDwo&s=default)

## Lifecycle
    https://www.processon.com/view/link/5dc7d645e4b0ffd21442c71a

## State
1. when start connection, save connection info
2. when start stream add add to channel
3. when broadcast send message to all channel's connection
4. when stop strem remove channel
5. when connection disconnected, update connection related channel

### Record
1. connections
   1. create
   2. delete
   3. find
   4. find connection's streams
   5. add stream to connection
   6. delete stream from connection
2. connections.stream
   name, channel pair
3. streams
   1. create
   2. delete
   3. find streams
4. streams.stream
   addr, channel pair
## The message protocal

## module
1. ws server
2. redis subscriber
3. rpc handler

## Channel message
### data struct
    1. connections: map addr -> connection
    2. streams: map stream -> subscriber
    3. subscribers: map subscriber -> streams
       for handle unsubscribe
    4. channels: map addr -> channels
       1. add
       2. remove


### subscribe
    client -> ws_server <-> app_server
    update channel_streams

### broadcast
    app server -> redis channel -> ws server

    find stream's addrs and send msg
