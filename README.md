# sls-releases
Service to collect last releases for multiproject repo.

Setup:
```shell
sdk use java 17.0.1-open
sdk use gradle 8.6
gradle publishImageToLocalRegistry
TOKEN=... docker-compose up
curl 'http://0.0.0.0:8080/sls/releases?rc=true'
```