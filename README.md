# sls-releases
Service to collect last releases for multiproject repo.

Setup:
```shell
sdk use java 17.0.1-open
sdk use gradle 8.6
gradle publishImageToLocalRegistry
TOKEN=your_token_here docker-compose up
```
or
```shell
sdk use java 17.0.1-open
sdk use gradle 8.6
GITHUB_TOKEN=your_token_here gradle run
```
then check
```shell
curl 'http://0.0.0.0:8080/sls/releases?rc=true'
```
or just
```shell
TOKEN=your_token_here just run
```