# MadProxy

Rust Proxy is an Easy and Fast Reverse Proxy for the Minecraft Java Community.
It's reliable and gives easy domains for users to remmember the servers and share it accross there players ans friend.

### Internal Documentation
- ``config.yml`` -> Contains all Config realated to the proxy.
- ``servers.json`` -> Containers all the servers that the proxy will reverse.


``config.yml``
```yml
---
listen_addr: "0.0.0.0:25565"
unknown_host:
  kick_message: "§6§lMadProxy\n§c§lServer is not available\n §7Please try again later\n\n §7Contact an administrator if the issue persists"
  motd:
    text: "§c§lServer does not exist §r\n§l§7AD: §fGet free 24/7 hosting @ §2xeh6.co.uk"
    protocol_name: "§cNot Found"
offline_server:
  kick_message: "§6§lMadProxy\n§c§lServer is currently offline\n §7Please try again later\n\n §7Contact an administrator if the issue persists"
  starting_message: "§6§lMadProxy\n§e§lServer is starting...\n §7Please try again in a moment"
  motd:
    text: "§c§lServer is offline §r\n§l§7AD: §fGet free 24/7 hosting @ §2xeh6.co.uk"
    protocol_name: "§cServer Offline"
auto_start: false
panel_link: "https://panel.novakraft.net"
api_key: "ptlc_Dweana6FNGD5XnSgKKXWLbpM29gCDp51j7ddhe6tH0A"

```

``servers.json``
```json
[
    {
        "id": "01cd2223",
        "hostname": "join.nyxalis.xyz",
        "backendServer": "panel.novakraft.net:15577"
    },
    {
        "id": "1d38d999",
        "hostname": "play.nyxalis.xyz",
        "backendServer": "panel.novakraft.net:15565"
    }
]
```