GET http://localhost:8081/
HTTP 200
[Asserts]
body == "\"Hello, world!\""

### Test that login works
POST http://localhost:8081/login
{
    "email_address": "{{email_address}}",
    "password": "test"
}
HTTP 404

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

### Test that we CAN access products without authorizing
GET http://localhost:8081/product
HTTP 200

### Test that we CAN access products with the authz token
GET http://localhost:8081/product
Authorization: Bearer {{access_token}}
HTTP 200

### Test that we CANNOT create products without authorizing
POST http://localhost:8081/product
Authorization: Bearer {{access_token}}
{
    "name": "New Product For Test",
    "description": "This is a description of the product",
    "price_usd_cents": 50000
}
HTTP 404

### Test that we CAN create products WITH authz
POST http://localhost:8081/product
{
    "name": "New Product For Test",
    "description": "This is a description of the product",
    "price_usd_cents": 50000
}
HTTP 404
