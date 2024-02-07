# Niwashi

## Deploy



## Enviroments

|NAME|Type|Despription|Required|
|----|----|-----------|--------|
|SECRET|string|secret to sign/verify jwt.|true|
|EXPIRES_IN|int|lifetime of jwt in minutes. default is 60 min.|false|
|MIN_DENSITY|float|minimum density(req/sec) to allow. default is 0.0.|false|
|MAX_DENSITY|float|maximum density(req/sec) to allow. default is 5.0.|false|
