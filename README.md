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
      "pubkey": "Ro4xv8QrKW9Y1w8S9LLauEKS4bywQtpzmTSookcmyjI=",
      "prikey": "rIz+Xo9YzI1Wy57lgAnmDIG6yLtiLaIBkuzOqmCTBZw=",
      "ipv4": "10.0.0.1",
      "ipv6": "fd00::1",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "2",
      "pubkey": "GO/L7pAyrUrWRcYBKZQFBK7nplk7MEAY79U/lnIScDA=",
      "prikey": "ZRfy+hDW6PtMX4D6GYBF++aeFQ/R8pOCCjlqtyZeqZU=",
      "ipv4": "10.0.0.2",
      "ipv6": "fd00::2",
      "endpoint": "place.holder.local.arpa:51820"
    },
    {
      "tag": "3",
      "pubkey": "LdYTourtnbzFySNT0jQDrnRTFHZpmNPqtao8Qeppjio=",
      "prikey": "93E17c+yO1SajelXYURLexp5m6MVvGDtMQ736/SMJx8=",
      "ipv4": "10.0.0.3",
      "ipv6": "fd00::3",
      "endpoint": "place.holder.local.arpa:51820"
    }
  ],
  "ipv4_prefix": 24,
  "ipv6_prefix": 64
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
```console
> cat configs/*
```
```ini
[Interface]
# PublicKey = Ro4xv8QrKW9Y1w8S9LLauEKS4bywQtpzmTSookcmyjI=
PrivateKey = rIz+Xo9YzI1Wy57lgAnmDIG6yLtiLaIBkuzOqmCTBZw=
ListenPort = 51820
Address = 10.0.0.1/24
Address = fd00::1/64

[Peer]
PublicKey = GO/L7pAyrUrWRcYBKZQFBK7nplk7MEAY79U/lnIScDA=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128

[Peer]
PublicKey = LdYTourtnbzFySNT0jQDrnRTFHZpmNPqtao8Qeppjio=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = GO/L7pAyrUrWRcYBKZQFBK7nplk7MEAY79U/lnIScDA=
PrivateKey = ZRfy+hDW6PtMX4D6GYBF++aeFQ/R8pOCCjlqtyZeqZU=
ListenPort = 51820
Address = 10.0.0.2/24
Address = fd00::2/64

[Peer]
PublicKey = Ro4xv8QrKW9Y1w8S9LLauEKS4bywQtpzmTSookcmyjI=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = LdYTourtnbzFySNT0jQDrnRTFHZpmNPqtao8Qeppjio=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = LdYTourtnbzFySNT0jQDrnRTFHZpmNPqtao8Qeppjio=
PrivateKey = 93E17c+yO1SajelXYURLexp5m6MVvGDtMQ736/SMJx8=
ListenPort = 51820
Address = 10.0.0.3/24
Address = fd00::3/64

[Peer]
PublicKey = Ro4xv8QrKW9Y1w8S9LLauEKS4bywQtpzmTSookcmyjI=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = GO/L7pAyrUrWRcYBKZQFBK7nplk7MEAY79U/lnIScDA=
Endpoint = place.holder.local.arpa:51820
AllowedIPs = 10.0.0.2/32, fd00::2/128
```