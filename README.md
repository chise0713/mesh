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
      "pubkey": "0tnoda1jLLX4bWDmoW19IfE7jPOgMv7i9jr50JLvthI=",
      "prikey": "uQB1T1YCgbiHbzCKGl8eFbUG0yTk9WEHa2y5blbhJMU=",
      "ipv4": "10.0.0.1",
      "ipv6": "fd00::1"
    },
    {
      "tag": "2",
      "pubkey": "5KC3jUkpMlnNZdP0wIts5i63X4HoA1qjsC3wJwiIzGg=",
      "prikey": "7njxUbzu0+1QLoH0MKt/jS4wjAvckfJPWV2siulaIrA=",
      "ipv4": "10.0.0.2",
      "ipv6": "fd00::2"
    },
    {
      "tag": "3",
      "pubkey": "rfzedR3Ya9+7ofGpJSzzyFGSESBVPILWp/OfrtSV/hQ=",
      "prikey": "PL1BkhgWBdWfLsS5ty/RX2NuL7OZ1jIIXiiSOBeDF4Q=",
      "ipv4": "10.0.0.3",
      "ipv6": "fd00::3"
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
# PublicKey = 0tnoda1jLLX4bWDmoW19IfE7jPOgMv7i9jr50JLvthI=
PrivateKey = uQB1T1YCgbiHbzCKGl8eFbUG0yTk9WEHa2y5blbhJMU=
Address = 10.0.0.1/24
Address = fd00::1/64


[Peer]
PublicKey = 5KC3jUkpMlnNZdP0wIts5i63X4HoA1qjsC3wJwiIzGg=
AllowedIPs = 10.0.0.2/32, fd00::2/128

[Peer]
PublicKey = rfzedR3Ya9+7ofGpJSzzyFGSESBVPILWp/OfrtSV/hQ=
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = 5KC3jUkpMlnNZdP0wIts5i63X4HoA1qjsC3wJwiIzGg=
PrivateKey = 7njxUbzu0+1QLoH0MKt/jS4wjAvckfJPWV2siulaIrA=
Address = 10.0.0.2/24
Address = fd00::2/64


[Peer]
PublicKey = 0tnoda1jLLX4bWDmoW19IfE7jPOgMv7i9jr50JLvthI=
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = rfzedR3Ya9+7ofGpJSzzyFGSESBVPILWp/OfrtSV/hQ=
AllowedIPs = 10.0.0.3/32, fd00::3/128
[Interface]
# PublicKey = rfzedR3Ya9+7ofGpJSzzyFGSESBVPILWp/OfrtSV/hQ=
PrivateKey = PL1BkhgWBdWfLsS5ty/RX2NuL7OZ1jIIXiiSOBeDF4Q=
Address = 10.0.0.3/24
Address = fd00::3/64


[Peer]
PublicKey = 0tnoda1jLLX4bWDmoW19IfE7jPOgMv7i9jr50JLvthI=
AllowedIPs = 10.0.0.1/32, fd00::1/128

[Peer]
PublicKey = 5KC3jUkpMlnNZdP0wIts5i63X4HoA1qjsC3wJwiIzGg=
AllowedIPs = 10.0.0.2/32, fd00::2/128
```