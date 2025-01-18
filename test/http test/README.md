1. Go to the Rust server frontend
```http
GET https://axum-jwt-static-page-template-4gs7.shuttle.app/ HTTP/1.1
content-type: application/json
```

2. Get all operations on a particular Rover (here eg:- Rover 2) -->> Akmal/Chamath
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/rover/operations/2 HTTP/1.1
```

3. Get the current operation status of a particular Rover (here eg:- Rover 1) -->> Akmal/Chathupa
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/rover/status/1 HTTP/1.1
```

4. Add new operation for a particular Rover (This will not working if DB and Image processing Server are not working)
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/rover HTTP/1.1
content-type: application/json

{
  "roverId":1 ,
  "randomId":1234 ,
  "batteryStatus":12.3 ,
  "temp": 12.3,
  "humidity": 12.3,
  "imageData": {
    "mime": "image/png",
    "data": "test"
  }
}
```

5. Mock rover result
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/test/rover HTTP/1.1
content-type: application/json

{
  "roverId":1 ,
  "randomId":1234 ,
  "batteryStatus":12.3 ,
  "temp": 12.3,
  "humidity": 12.3,
  "imageData": {
    "mime": "image/png",
    "data": "test"
  }
}
```

6. This will add a new Rover to Postgres DB(When a new rover is created on FastAPI server this also need to run) -->> Akmal/Chamath
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/rover/new HTTP/1.1
content-type: application/json

{
    "roverId":12345,
    "initialId":12345,
    "roverStatus":12345,
    "userId":12345
}
```

7. Update rover `initialId` and `roverStatus` for given `userId`. -->> Akmal/Chamath/Chathupa
```http
POST https://axum-jwt-static-page-template-4gs7.shuttle.app/rover/update HTTP/1.1
content-type: application/json

{
    "initialId": 12345,
    "roverStatus": 1,
    "userId": 1
}
```

8. Get only state of a Rover given.(here eg:- Rover 3) -->> Akmal/Chamath/Chathupa
```http
GET https://axum-jwt-static-page-template-4gs7.shuttle.app/api/user/1
content-type: application/json
```
