Example config:

```toml
[debug]
address = "127.0.0.1"
port = 8000
workers = 1
keep_alive = 30
log = "info"

secret_key = "" # 32 bytes hex string

oauth2_url = ""
client_id = 0
client_secret = ""
redirect_uri = ""

[release]
# same fields as debug
```
