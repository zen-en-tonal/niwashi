# Niwashi

## Deploy

```console
docker container run -e SECRET=secret -p 3000:3000 niwashi:latest
```

## Enviroments

|NAME|Type|Despription|Required|
|----|----|-----------|--------|
|SECRET|string|secret to sign/verify jwt.|true|
|EXPIRES_IN|int|lifetime of jwt in minutes. default is 60 min.|false|
|MIN_DENSITY|float|minimum density(req/sec) to allow. default is 0.0.|false|
|MAX_DENSITY|float|maximum density(req/sec) to allow. default is 5.0.|false|
|PORT|int|port to expose the service. default is 3000.|false|
