# Mesh
```console
WireGuard Mesh Configuration File Generator

Usage: mesh --config <CONFIG> <COMMAND>

Commands:
  init     Init a mesh config file
  convert  Convert mesh config to wireguard config
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
      "pubkey": "YTLfTezjWxunlJjwBbSEBJGhsj6M0rfbore8rbV/tW4=",
      "prikey": "W1ytmL9FPm0o/l5047TVY4fML4tpRVTRRmej4J3liqQ=",
      "ipv4": "10.0.0.1",
      "ipv6": "fd00::1",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "2",
      "pubkey": "/M5I2YBMTCbMNkIx1hSECkogN+JmW+cPf9aNWX+EGEs=",
      "prikey": "/OmWhS9gyqOgrxzrxQhBPGNQFpWlaykEnsnBuyfaHzo=",
      "ipv4": "10.0.0.2",
      "ipv6": "fd00::2",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "3",
      "pubkey": "PL7TVmNXV/M5VnRPNWlMTBgxawD+fGAB2EMEooI1m1Y=",
      "prikey": "ycqBsFj+hjxJXLcMx5Hgo3F0ZiM+YDMfcJNYlZn+HQ4=",
      "ipv4": "10.0.0.3",
      "ipv6": "fd00::3",
      "endpoint": "place.holder.local.arpa:51820"
    }
  ],
  "ipv4_prefix": 24,
  "ipv6_prefix": 64
}⏎          
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
```console
> cat configs/*
```
```ini
[Interface]
# PublicKey = YTLfTezjWxunlJjwBbSEBJGhsj6M0rfbore8rbV/tW4=
PrivateKey = W1ytmL9FPm0o/l5047TVY4fML4tpRVTRRmej4J3liqQ=
Address = 10.0.0.1/24
Address = fd00::1/64

[Peer]
PublicKey = /M5I2YBMTCbMNkIx1hSECkogN+JmW+cPf9aNWX+EGEs=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128

[Peer]
PublicKey = PL7TVmNXV/M5VnRPNWlMTBgxawD+fGAB2EMEooI1m1Y=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = /M5I2YBMTCbMNkIx1hSECkogN+JmW+cPf9aNWX+EGEs=
PrivateKey = /OmWhS9gyqOgrxzrxQhBPGNQFpWlaykEnsnBuyfaHzo=
Address = 10.0.0.2/24
Address = fd00::2/64

[Peer]
PublicKey = YTLfTezjWxunlJjwBbSEBJGhsj6M0rfbore8rbV/tW4=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = PL7TVmNXV/M5VnRPNWlMTBgxawD+fGAB2EMEooI1m1Y=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = PL7TVmNXV/M5VnRPNWlMTBgxawD+fGAB2EMEooI1m1Y=
PrivateKey = ycqBsFj+hjxJXLcMx5Hgo3F0ZiM+YDMfcJNYlZn+HQ4=
Address = 10.0.0.3/24
Address = fd00::3/64

[Peer]
PublicKey = YTLfTezjWxunlJjwBbSEBJGhsj6M0rfbore8rbV/tW4=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = /M5I2YBMTCbMNkIx1hSECkogN+JmW+cPf9aNWX+EGEs=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128
```