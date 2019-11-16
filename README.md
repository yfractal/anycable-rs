[WIP] Anycable Rust WebSocket server

## Dev
1. start ws server
   `cargo run`
2. start grpc sever and app server
   go to anycable_demo directory
   run grpc server:
   `ADAPTER=any_cable ANYCABLE_DEBUG=1 bundle exec anycable`
   run app server:
   `ADAPTER=any_cable CABLE_URL='ws://localhost:3334/ws/cable' bundle exec rails s -b 0.0.0.0`

   run erlang ws server
   go to erlycable directory
   `rebar3 shell`

### Dev utils
    `wscat -c ws://localhost:3334/ws/cable`

### User Story Base on anycable_demo
When login as tom, then should receive welcome message({"type":"welcome"}).

When send subscribe message through ws
`{"command":"subscribe","identifier":"{\"channel\":\"NotificationChannel\",\"id\":\"edmund_hintz\"}"}`

Then should receive confirm message
`{"identifier":"{\"channel\":\"NotificationChannel\",\"id\":\"edmund_hintz\"}","type":"confirm_subscription"}`

When follow the channel
`{"identifier":"{\"channel\":\"NotificationChannel\",\"id\":\"edmund_hintz\"}","type":"confirm_subscription"}`

Then should receive follow confirm message
`{"identifier":"{\"channel\":\"NotificationChannel\",\"id\":\"edmund_hintz\"}","type":"confirm_subscription"}`

Then should receive ping message
`{"type":"ping","message":1573696065072}`

When create buckets through api,
http://localhost:3000/baskets

basket[name]: bucket
basket[description]: abcd
commit: Save

Then should receive backet info by ws
`{"identifier":"{\"channel\":\"BasketsChannel\"}","message":{"type":"create","data":{"id":4,"name":"bucket","logo_path":"https://unsplash.it/400/200?image=471","description":"abcd","owner":"edmund_hintz","items_count":0}}}`

When close ws connection

The server should delete all the connection's info.
