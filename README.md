# sls-releases
Service to collect last releases for multiproject repo.

Setup:
```shell
rustc --version
cargo --version

# 1) Fill token in ./application.toml:
#    github.token = "your_token_here"
#
# 2) run
cargo run
```
then check
```shell
curl 'http://0.0.0.0:8080/sls/releases?rc=true'
```

HTML response:
```shell
curl -H 'Accept: text/html' 'http://0.0.0.0:8080/sls/releases?rc=true'
```