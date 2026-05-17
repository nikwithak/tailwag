GET http://localhost:8081/
HTTP 200
[Asserts]
body == "\"Hello, world!\""

### Test that login works
# Note that I need to REMOVE the "null" check in favor of an actual 404
POST http://localhost:8081/login
{
    "email_address": "{{email_address}}",
    "password": "test"
}
HTTP 200
[Asserts]
body == "null"

### Test that registration of a new account works
POST http://localhost:8081/register
{
    "email_address": "{{email_address}}",
    "password": "test"
}
HTTP 200

### Test that login works for the new account
POST http://localhost:8081/login
{
    "email_address": "{{email_address}}",
    "password": "test"
}
HTTP 200
[Captures]
access_token: jsonpath "$['access']"

### Test that we can't access Events without authorizing
GET http://localhost:8081/event
HTTP 200

### Test that we CAN access events with the authz token
GET http://localhost:8081/event
Authorization: Bearer {{access_token}}
HTTP 200