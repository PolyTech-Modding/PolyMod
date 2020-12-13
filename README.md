Example config:

```toml
[debug]
address = "127.0.0.1"
port = 8000
workers = 1
keep_alive = 30
log = "info"

# You can generate these with this:
# https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=b5d238039658071a83b21ce6b5e4d99a
secret_key = "" # 32 bytes hex string
iv_key = "" # 16 bytes hex string

oauth2_url = ""
client_id = 0
client_secret = ""
redirect_uri = ""

redis_uri = "127.0.0.1:6379"
mods_path = "./tmp" # mkdir tmp

[release]
# same fields as debug
```
