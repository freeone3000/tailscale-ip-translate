# tailscale-ip-translate

Translates IPv4 addresses for a tailscale 6to4 subnet router.

See [the tailscale documentation](https://tailscale.com/kb/1201/4via6-subnets) for more information
as to what a 4via6 subnet router is, and why you would want one.

This tool aims to simplify the transition, allowing you to type in the ipv4 address, and retrieving
the ipv6 address for your tailnet.

## Installation
Installation can be performed via `cargo install tailscale-ip-translate`


## Usage
```shell
tailscale-ip-translate
tailscale-ip-translate <ipv4>
tailscale-ip-translate <site-id> <ipv4>
tailscale-ip-translate -r <ipv6>
```

If zero arguments are supplied, the IPv6 address for `192.168.1.0` on site 7 will be output.

If one argument is supplied, the IPv6 address for the supplied IPv4 address on site 7 will be output.

If two arguments are supplied, the IPv6 address for the supplied IPv4 address on the supplied site id will be output.

If the `-r` flag is supplied, the reverse translation is performed, taking an IPv6 address and outputting
the corresponding IPv4 address. The site id may be provided, but will be ignored.