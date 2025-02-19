# Mesh
```console
WireGuard Mesh Configuration File Generator

Usage: mesh --config <CONFIG> <COMMAND>

Commands:
  init     Init a mesh config file
  convert  Convert mesh config to wireguard config
  append   Append a `mesh` to the config
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Config file path
  -h, --help             Print help
  -V, --version          Print version
```

## Example
### Init
```shell
cargo run -- --config config.json init -c 3
```
```console
cat config.json
```
```json
{
  "meshs": [
    {
      "tag": "1",
      "pubkey": "+TA+VKmkOUIFIkZXUPL5qHRXJolqrUota5LMkcUkTjY=",
      "prikey": "6NHao92+e5vVxtHvr6uIjaPklyaRBUqKT5+p1UdBGwk=",
      "ipv4": "10.0.0.1",
      "ipv6": "fd00::1",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "2",
      "pubkey": "sQRrFafoEBtCNOCcLUqveXCVWgWNNdGPfyU6NuHUfH0=",
      "prikey": "VudTihfJQ1HAgleMp8ia3Brnqv3e7lRPexPmqXkXbPQ=",
      "ipv4": "10.0.0.2",
      "ipv6": "fd00::2",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "3",
      "pubkey": "PQrsML8xSQyJo91Y4RnCV66cJtkPuiFaY3OkyQIe7XE=",
      "prikey": "RX+pUaq25AVZHTw4+loXdEw/tPi895W87l3V9Pbv8ps=",
      "ipv4": "10.0.0.3",
      "ipv6": "fd00::3",
      "endpoint": "place.holder.local.arpa:51820"
    }
  ],
  "ipv4_prefix": 29,
  "ipv6_prefix": 126
}   
```
### Convert
```shell
mkdir configs
cargo run -- --config config.json convert -o configs/
```
```console
> ls configs/
1.conf  2.conf  3.conf
```
```ini
> cat configs/*
[Interface]
# PublicKey = +TA+VKmkOUIFIkZXUPL5qHRXJolqrUota5LMkcUkTjY=
PrivateKey = 6NHao92+e5vVxtHvr6uIjaPklyaRBUqKT5+p1UdBGwk=
ListenPort = 51820
Address = 10.0.0.1/29
Address = fd00::1/126

[Peer]
PublicKey = sQRrFafoEBtCNOCcLUqveXCVWgWNNdGPfyU6NuHUfH0=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128

[Peer]
PublicKey = PQrsML8xSQyJo91Y4RnCV66cJtkPuiFaY3OkyQIe7XE=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = sQRrFafoEBtCNOCcLUqveXCVWgWNNdGPfyU6NuHUfH0=
PrivateKey = VudTihfJQ1HAgleMp8ia3Brnqv3e7lRPexPmqXkXbPQ=
ListenPort = 51820
Address = 10.0.0.2/29
Address = fd00::2/126

[Peer]
PublicKey = +TA+VKmkOUIFIkZXUPL5qHRXJolqrUota5LMkcUkTjY=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = PQrsML8xSQyJo91Y4RnCV66cJtkPuiFaY3OkyQIe7XE=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = PQrsML8xSQyJo91Y4RnCV66cJtkPuiFaY3OkyQIe7XE=
PrivateKey = RX+pUaq25AVZHTw4+loXdEw/tPi895W87l3V9Pbv8ps=
ListenPort = 51820
Address = 10.0.0.3/29
Address = fd00::3/126

[Peer]
PublicKey = +TA+VKmkOUIFIkZXUPL5qHRXJolqrUota5LMkcUkTjY=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = sQRrFafoEBtCNOCcLUqveXCVWgWNNdGPfyU6NuHUfH0=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128
```
### Append
```shell
cargo run -- --config config.json append -t append
```
```json
{
  "meshs": [
    {
      "tag": "1",
      "pubkey": "+TA+VKmkOUIFIkZXUPL5qHRXJolqrUota5LMkcUkTjY=",
      "prikey": "6NHao92+e5vVxtHvr6uIjaPklyaRBUqKT5+p1UdBGwk=",
      "ipv4": "10.0.0.1",
      "ipv6": "fd00::1",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "2",
      "pubkey": "sQRrFafoEBtCNOCcLUqveXCVWgWNNdGPfyU6NuHUfH0=",
      "prikey": "VudTihfJQ1HAgleMp8ia3Brnqv3e7lRPexPmqXkXbPQ=",
      "ipv4": "10.0.0.2",
      "ipv6": "fd00::2",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "3",
      "pubkey": "PQrsML8xSQyJo91Y4RnCV66cJtkPuiFaY3OkyQIe7XE=",
      "prikey": "RX+pUaq25AVZHTw4+loXdEw/tPi895W87l3V9Pbv8ps=",
      "ipv4": "10.0.0.3",
      "ipv6": "fd00::3",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "append",
      "pubkey": "nVR7kP0cdg6jFgPbmnDkWZiO63/Gul7W1nJNyNXDy2g=",
      "prikey": "teSVJOS2+D9Df71ad+ccvEA6JpXAzigZIX7oXByGfK4=",
      "ipv4": "10.0.0.4",
      "ipv6": "fd00::4",
      "endpoint": "place.holder.local.arpa:51820"
    }
  ],
  "ipv4_prefix": 29,
  "ipv6_prefix": 125
}
```